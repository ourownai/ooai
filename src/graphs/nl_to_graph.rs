use pyo3::prelude::*;
use crate::bindings::spacy_bindings::{self, Doc, EntityLabel, TokenPos};
use std::collections::HashMap;
use reqwest::blocking::Client;
use serde_json::Value;

// Define the EntityGraph trait
pub trait EntityGraph {
    fn new() -> Self;
    fn add_entity(&mut self, entity_type: EntityType, entity_value: String);
    fn has_entities_of_type(&self, entity_type: &EntityType) -> bool;
    fn get_entities_of_type(&self, entity_type: &EntityType) -> Option<&Vec<String>>;
    fn merge(&mut self, other: Self);
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum EntityType {
    Location,
    Person,
    Organization,
    // Add more entity types as needed
}

// Define the EntityGraphImpl struct
pub struct EntityGraphImpl {
    entities: HashMap<EntityType, Vec<String>>,
}

impl EntityGraph for EntityGraphImpl {
    // Create a new empty EntityGraphImpl
    fn new() -> Self {
        EntityGraphImpl {
            entities: HashMap::new(),
        }
    }

    // Add an entity to the graph
    fn add_entity(&mut self, entity_type: EntityType, entity_value: String) {
        self.entities
            .entry(entity_type)
            .or_insert_with(Vec::new)
            .push(entity_value);
    }

    // Check if the graph contains entities of a specific type
    fn has_entities_of_type(&self, entity_type: &EntityType) -> bool {
        self.entities.contains_key(entity_type)
    }

    // Get the entities of a specific type
    fn get_entities_of_type(&self, entity_type: &EntityType) -> Option<&Vec<String>> {
        self.entities.get(entity_type)
    }

    // Merge another EntityGraph into the current one
    fn merge(&mut self, other: Self) {
        for (entity_type, entities) in other.entities {
            for entity_value in entities {
                self.add_entity(entity_type.clone(), entity_value);
            }
        }
    }
}

// Function to parse a message and extract entities into an EntityGraph
pub fn parse_message(doc: &Doc) -> impl EntityGraph {
    let mut entity_graph = EntityGraphImpl::new();

    // Acquire the Python GIL
    Python::with_gil(|py| {
        // Process entities found in the message
        for ent in doc.ents(py).unwrap() {
            let entity_type = match ent.export(py).unwrap().label.to_string().as_str() {
                "PERSON" => EntityType::Person,
                "ORG" => EntityType::Organization,
                "GPE" => EntityType::Location,
                _ => continue,
            };
            let entity_value = ent.text(py).unwrap();
            entity_graph.add_entity(entity_type, entity_value);
        }
    });

    entity_graph
}

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

    // Merges another QueryMapping into the current one
    fn merge(&mut self, other: QueryMapping) {
        self.entity_map.extend(other.entity_map);
        self.slot_map.extend(other.slot_map);
    }
}

// Converts an utterance into a QueryMapping struct, extracting entities and slots
fn utterance_to_query_mapping(utterance: &str) -> QueryMapping {
    let doc = spacy_bindings::SPACY.model_default().nlp(utterance.to_string()).unwrap();
    let entity_mapping = get_entity_mapping();
    let mut mapping = QueryMapping::new();

    // Process entities found in the utterance
    for ent in doc.ents(doc.object).unwrap() {
        if let Some(&entity_field) = entity_mapping.get(&ent.export(doc.object).unwrap().label.to_string()) {
            mapping.add_entity(entity_field.to_string(), ent.text(doc.object).unwrap());
        }
    }

    // Process tokens to identify and add slot values
    process_tokens_for_slots(&doc, &mut mapping);

    mapping
}

// Returns a mapping of spaCy entity labels to GraphQL query fields
fn get_entity_mapping() -> HashMap<String, &'static str> {
    [
        (EntityLabel::Person.to_string(), "person"),
        (EntityLabel::Org.to_string(), "organization"),
        (EntityLabel::Gpe.to_string(), "location"),
        (EntityLabel::Money.to_string(), "amount"),
    ]
    .iter()
    .cloned()
    .collect()
}

// Identifies slots from tokens and adds them to the mapping
fn process_tokens_for_slots(doc: &Doc, mapping: &mut QueryMapping) {
    for token in doc.tokens(doc.object).unwrap().iter().filter(|t| t.pos != TokenPos::PUNCT) {
        let token_text = &token.text;
        let token_lower = token_text.to_lowercase();
        let token_pos = token.pos;

        if matches!(token_pos, TokenPos::NOUN | TokenPos::PROPN) {
            mapping.add_slot(token_lower, token_text.to_string());
        } else if token_pos == TokenPos::VERB {
            if let Some(obj) = token.children(doc.object).unwrap().iter().find(|c| c.dep(doc.object).unwrap() == "dobj") {
                let slot_value = obj.text.to_lowercase();
                mapping.add_slot(slot_value.clone(), obj.text.to_string());
            }
        } else if token_pos == TokenPos::ADJ {
            if let Some(noun) = token.head(doc.object).unwrap().children(doc.object).unwrap().iter().find(|c| c.pos == TokenPos::NOUN) {
                let slot_value = noun.text.to_lowercase();
                mapping.add_slot(slot_value.clone(), noun.text.to_string());
            }
        }
    }
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
        query.push_str(&format!(
            "{}(name: \"{}\") {{ id name {} }} ",
            entity_field, entity_value, entity_field
        ));
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

// Parses the response JSON and extracts relevant information
fn parse_response(response: &str) -> Result<String, Box<dyn std::error::Error>> {
    let json: Value = serde_json::from_str(response)?;
    // Assuming the response has a "data" field containing the query results
    if let Some(data) = json.get("data") {
        // Extract and format the relevant information from the response
        // This part will depend on the structure of your GraphQL response
        let formatted_data = format!("{:#?}", data);
        Ok(formatted_data)
    } else {
        Err("Invalid response format".into())
    }
}

fn main() {
    let utterance = "I want to pay Bob $50"; // Example utterance
    let mapping = utterance_to_query_mapping(utterance); // Convert utterance to QueryMapping

    match generate_query_from_mapping(&mapping) {
        // Generate and send the query, then handle the response
        Ok(response) => {
            println!("Response: {}", response);
            match parse_response(&response) {
                Ok(parsed_data) => println!("Parsed Data: {}", parsed_data),
                Err(e) => eprintln!("Error parsing response: {}", e),
            }
        }
        Err(e) => eprintln!("Error generating query: {}", e),
    }
}
