/*
This module introduces a set of functions and structs that collectively enable the automatic creation and suggestion of relevant chart types based on user inputs and data schemas. The core functionality revolves around leveraging Natural Language Processing (NLP) techniques in conjunction with knowledge graphs to interpret user utterances and map them to appropriate chart configurations.

Key components and functionalities of this module include:

QueryMapping struct: This central data structure encapsulates the mappings between entities and slots identified in the user's utterance and their corresponding values. The entity_map field associates recognized entity types (e.g., chart types) with their specific instances, while the slot_map field maps extracted slots (e.g., data fields) to their respective values. These mappings facilitate the construction of a structured query to retrieve relevant chart information from a knowledge graph.
utterance_to_query_mapping function: This function serves as the entry point for processing user utterances. It utilizes the spaCy NLP model to analyze the input utterance, extracting entities and slots based on predefined mappings and linguistic patterns. The extracted information is encapsulated within a QueryMapping instance, which serves as the foundation for subsequent query generation and chart suggestion steps.
generate_query_from_mapping function: This function constructs a GraphQL query string based on the QueryMapping instance obtained from the user's utterance. It leverages the construct_query_string function to build the query by incorporating the mapped entities and slots. The generated query is then sent to a GraphQL endpoint using the send_query function, which handles the communication with the knowledge graph and retrieves the relevant chart information.
prepare_data_for_chart function: Once the suggested chart type and associated data fields are obtained from the GraphQL response, this function prepares the data for visualization. It utilizes the DataBin and ChartPlot structs to organize and structure the data according to the requirements of the selected chart type. The prepared data is then ready to be passed to the charting library for rendering.
bokeh_bindings::plot_figure function: This function serves as the interface to the Bokeh charting library. It receives the prepared data, suggested chart type, and chart configuration settings, and generates the corresponding chart using Bokeh's plotting capabilities. The resulting chart provides a visual representation of the data based on the user's expressed preferences and the underlying data schema.
The main flow of the module involves the following steps:

Accepting a user utterance expressing the desired chart type and data fields.
Processing the utterance using NLP techniques to extract entities and slots.
Generating a GraphQL query based on the extracted information and retrieving chart suggestions from a knowledge graph.
Preparing the data for the suggested chart type using the available data schema.
Configuring the chart's appearance and plotting it using the Bokeh library.
By combining NLP, knowledge graphs, and data visualization techniques, this module provides a powerful and flexible framework for automating the creation and suggestion of relevant charts based on user inputs and data schemas. It streamlines the process of translating user requirements into meaningful visual representations, enhancing data exploration and decision-making capabilities.
*/

use std::collections::HashMap;
use reqwest::blocking::Client;
use serde_json::Value;
use pyo3::Python;

use crate::bindings::bokeh_charts::prepare_data_for_chart;
use crate::bindings::bokeh_bindings::{DataBin, ChartConfig, plot_figure, find_group, group_by, ticker, group_commons, array_count, array_sum, array_average, linear_scale_mixin};
use crate::bindings::spacy_bindings::{BigbotError, Doc, SpacyModule, TokenPos};


// To integrate with charting functionality
// Define a struct to hold the mappings of entities and slots identified in an utterance
#[derive(Debug)]
struct QueryMapping {
    entity_map: HashMap<String, String>, // Maps recognized entities to their values
    slot_map: HashMap<String, String>,   // Maps identified slots to their values
}

impl QueryMapping {
    // Constructor for a new QueryMapping instance
    fn new() -> Self {
        Self {
            entity_map: HashMap::new(),
            slot_map: HashMap::new(),
        }
    }

    // Adds an identified entity and its value to the entity_map
    fn add_entity(&mut self, key: String, value: String) {
        self.entity_map.insert(key, value);
    }

    // Adds a detected slot and its value to the slot_map
    fn add_slot(&mut self, key: String, value: String) {
        self.slot_map.insert(key, value);
    }
}

// Converts an utterance into a QueryMapping struct, extracting entities and slots
fn utterance_to_query_mapping(utterance: &str) -> Result<QueryMapping, BigbotError> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let model = SpacyModule::model_default(py);
    let doc = model.nlp(utterance.to_string())?;
    let entity_mapping = get_entity_mapping(); // Get predefined entity to query field mappings
    let mut mapping = QueryMapping::new();

    // Process entities found in the utterance
    for ent in doc.ents(py)? {
        if let Some(&entity_field) = entity_mapping.get(&ent.label) {
            mapping.add_entity(entity_field.to_string(), ent.text(py)?.to_string());
        }
    }

    // Process tokens to identify and add slot values
    process_tokens_for_slots(&doc, &mut mapping, py)?;

    Ok(mapping)
}

// Returns a mapping of spaCy entity labels to GraphQL query fields
fn get_entity_mapping() -> HashMap<&'static str, &'static str> {
    [("CHART_TYPE", "chartType"), ("DATA_FIELD", "dataField")]
        .iter()
        .cloned()
        .collect()
}

// Identifies slots from tokens and adds them to the mapping
fn process_tokens_for_slots(doc: &Doc, mapping: &mut QueryMapping, py: Python) -> Result<(), BigbotError> {
    for token in doc.tokens(py)?.into_iter().filter(|t| t.pos != TokenPos::PUNCT) {
        let token_text = token.text.clone();
        let token_lower = token_text.to_lowercase();
        let token_pos = token.pos;

        if matches!(token_pos, TokenPos::NOUN | TokenPos::PROPN) {
            mapping.add_slot(token_lower, token_text);
        } else if token_pos == TokenPos::VERB {
            if let Some(obj) = token.children(py)?.into_iter().find(|c| c.dep(py)? == "dobj") {
                let slot_value = obj.text.to_lowercase();
                mapping.add_slot(slot_value.clone(), obj.text);
            }
        } else if token_pos == TokenPos::ADJ {
            if let Some(noun) = token.head(py)?.children(py)?.into_iter().find(|c| c.pos == TokenPos::NOUN) {
                let slot_value = noun.text.to_lowercase();
                mapping.add_slot(slot_value.clone(), noun.text);
            }
        }
    }

    Ok(())
}

// Generates a GraphQL query string from the QueryMapping and sends it to a GraphQL endpoint
fn generate_query_from_mapping(mapping: &QueryMapping) -> Result<String, Box<dyn std::error::Error>> {
    let query = construct_query_string(mapping); // Construct the GraphQL query string
    let response = send_query(&query)?; // Send the query and receive the response
    Ok(response)
}

// Constructs the GraphQL query string from the entity and slot mappings
fn construct_query_string(mapping: &QueryMapping) -> String {
    let mut query = String::from("query { ");

    // Construct query parts for entities
    construct_entity_queries(&mut query, mapping);

    // Construct query parts for slots
    construct_slot_queries(&mut query, mapping);

    query.push('}');
    query
}

// Adds query parts for each entity in the mapping to the query string
fn construct_entity_queries(query: &mut String, mapping: &QueryMapping) {
    for (entity_field, entity_value) in &mapping.entity_map {
        query.push_str(&format!("{}(name: \"{}\") {{ id name {} }} ", entity_field, entity_value, entity_field));
    }
}

// Adds query parts for each slot in the mapping to the query string
fn construct_slot_queries(query: &mut String, mapping: &QueryMapping) {
    // Iterate over each slot in the slot_map
    for (slot_name, slot_value) in &mapping.slot_map {
        // Assume slots are additional filters or query parameters for entities
        // This part may need adjustments based on the specific GraphQL schema and requirements
        // Check if the slot corresponds to any entity in the entity_map to avoid redundancy
        if !mapping.entity_map.contains_key(slot_name) {
            // Here we are adding the slot as a filter to the query.
            // Adjust the query structure as needed based on your GraphQL schema.
            // For example, this could be a generic filter applied to a specific entity type
            // or a way to add additional fields to the query based on the slot's context.
            query.push_str(&format!(
                "{{ filter: {{ {} : {{ eq: \"{}\" }} }} }} ",
                slot_name, slot_value
            ));
        }
    }
}

// Sends the constructed GraphQL query to the specified endpoint and returns the response
fn send_query(query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new(); // Initialize the HTTP client
    let url = "https://example.com/graphql"; // Endpoint URL
    let response = client.post(url).body(query.to_owned()).send()?; // Send the query

    if response.status().is_success() {
        Ok(response.text()?) // Return the response text if successful
    } else {
        Err("Failed to send query".into()) // Return an error if the request failed
    }
}

fn main() {
    // Example usage
    let data = vec![
        HashMap::from([
            ("category".to_string(), "A".to_string()),
            ("value1".to_string(), "10".to_string()),
            ("value2".to_string(), "20".to_string()),
        ]),
        HashMap::from([
            ("category".to_string(), "B".to_string()),
            ("value1".to_string(), "15".to_string()),
            ("value2".to_string(), "25".to_string()),
        ]),
        HashMap::from([
            ("category".to_string(), "C".to_string()),
            ("value1".to_string(), "20".to_string()),
            ("value2".to_string(), "30".to_string()),
        ]),
    ];
    let fields = vec!["category".to_string(), "value1".to_string(), "value2".to_string()];
    let mut data_bin = DataBin::new(data, fields);

    let utterance = "Show me a line chart for value1 and value2"; // Example utterance
    let mapping = utterance_to_query_mapping(utterance); // Convert utterance to QueryMapping

    match generate_query_from_mapping(&mapping) { // Generate and send the query, then handle the response
        Ok(query) => {
            println!("Query: {}", query);
            // Parse the response and extract the suggested chart type and data fields
            let chart_type = mapping.entity_map.get("chartType").cloned().unwrap_or_default();
            let data_fields: Vec<String> = mapping.slot_map.values().cloned().collect();

            // Prepare the data for the suggested chart type
            let chart_data = prepare_data_for_chart(&data_bin, &chart_type);

            // Create the chart configuration
            let mut chart_config = ChartConfig::new();
            chart_config.set_plot_width(800);
            chart_config.set_plot_height(600);

            // Plot the chart using Bokeh
            plot_figure(&data_bin, &chart_type, &chart_config);
        }
        Err(e) => eprintln!("Error generating query: {}", e),
    }

    // Example usage of find_group
    let groups = find_group(&data_bin.data, "category", false);
    println!("Groups: {:?}", groups);

    // Example usage of group_by
    let grouped_data = group_by(&data_bin.data, "category", false);
    println!("Grouped Data: {:?}", grouped_data);

    // Example usage of ticker
    let tick_value = 3.14159;
    let tick_string = ticker(tick_value);
    println!("Tick String: {}", tick_string);

    // Example usage of group_commons
    let common_groups = group_commons(&data_bin.data, "category", "value1", "value2");
    println!("Common Groups: {:?}", common_groups);

    // Example usage of array_count
    let array = vec![vec!["a".to_string(), "b".to_string()], vec!["c".to_string(), "d".to_string(), "e".to_string()]];
    let counts = array_count(&array);
    println!("Array Counts: {:?}", counts);

    // Example usage of array_sum
    let sums = array_sum(&array);
    println!("Array Sums: {:?}", sums);

    // Example usage of array_average
    let averages = array_average(&array);
    println!("Array Averages: {:?}", averages);

    // Example usage of linear_scale_mixin
    let array = vec![1, 2, 3, 4, 5];
    let scaled_array = linear_scale_mixin(&array, 2, false, 10);
    println!("Scaled Array: {:?}", scaled_array);
}