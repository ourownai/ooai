/*
This module defines a system for analysing and organising information from natural language input into a structured format, focusing on attributes and connections within a delegate model. It consists of two primary structures: `Attribute` and `Delegate`.

- `Attribute`: Represents a characteristic or quality of a delegate, identified by a name and associated with a set of values. This structure is designed to capture and store various attributes extracted from the input text, such as interests or expertise areas, in a way that facilitates easy addition and querying of attribute values.

- `Delegate`: Represents an entity with a collection of attributes and connections to other entities. The `Delegate` struct maintains a hashmap of attributes to efficiently store and access the delegate's characteristics. Additionally, it manages a hashmap of connections that represent relationships between different entities or concepts identified in the input text. These connections are dynamically built based on the context and content of the input.

Key Functionalities:
- `add_attribute_value`: Adds a new value to a specified attribute of the delegate. If the attribute does not exist, it is created.
- `has_attribute_value`: Checks if a specific value is associated with a given attribute, facilitating the validation of attribute contents.
- `build_network`: Processes an input string to extract and organize information into attributes and connections. This method first identifies and categorizes attribute values based on predefined prefixes (e.g., "interest:" or "expertise:"). It then scans the input to construct a network of connections between non-attribute entities, applying logic to identify relationships marked by specific tokens (e.g., "->").

Enhancements:
- The script has been refactored for improved modularity, splitting the network building process into separate, focused methods (`process_attributes` and `process_connections`) for clearer logic and better maintainability.
- The use of `HashMaps` and `HashSets` remains integral to the design, providing efficient storage and retrieval of attributes and connections. The refactoring enhances their usage, ensuring data is processed and organized in a more streamlined and readable manner.
- Optimizations include more efficient parsing strategies and the use of windowed iteration for connection processing, reducing complexity and improving performance.

This module demonstrates a practical application of data structures and algorithms in processing and structuring complex natural language data, suitable for applications in natural language understanding, information extraction, and knowledge graph construction.
*/

use std::collections::{HashMap, HashSet};
use std::error::Error;

#[derive(Debug)]
pub struct Attribute {
    pub name: String,
    pub values: HashSet<String>,
}

impl Attribute {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            values: HashSet::new(),
        }
    }

    pub fn add_value(&mut self, value: &str) {
        self.values.insert(value.to_string());
    }
}

#[derive(Debug)]
pub struct Delegate {
    pub attributes: HashMap<String, Attribute>,
    pub connections: HashMap<String, HashSet<String>>,
}

impl Delegate {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    pub fn add_attribute_value(&mut self, attr_name: &str, value: &str) {
        self.attributes
            .entry(attr_name.to_string())
            .or_insert_with(|| Attribute::new(attr_name))
            .add_value(value);
    }

    pub fn has_attribute_value(&self, attr_name: &str, value: &str) -> bool {
        self.attributes
            .get(attr_name)
            .map_or(false, |attr| attr.values.contains(value))
    }

    pub fn build_network(&mut self, input: &str) -> Result<(), Box<dyn Error>> {
        let tokens: Vec<&str> = input.split_whitespace().collect();
        self.process_attributes(&tokens);
        self.process_connections(&tokens);
        Ok(())
    }

    fn process_attributes(&mut self, tokens: &[&str]) {
        tokens.iter().for_each(|&token| {
            if let Some((attr_name, value)) = token.split_once(':') {
                self.add_attribute_value(attr_name, value);
            }
        });
    }

    fn process_connections(&mut self, tokens: &[&str]) {
        tokens.windows(3).for_each(|window| {
            if window[1] == "->" {
                let (head_word, dep_word) = (window[0], window[2]);
                if !self.has_attribute_value("interests", dep_word)
                    && !self.has_attribute_value("expertise", dep_word)
                {
                    self.connections
                        .entry(head_word.to_string())
                        .or_default()
                        .insert(dep_word.to_string());
                }
            }
        });
    }

    pub fn get_connections(&self, word: &str) -> Option<&HashSet<String>> {
        self.connections.get(word)
    }

    pub fn get_attributes(&self, attr_name: &str) -> Option<&Attribute> {
        self.attributes.get(attr_name)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "My interests are sports. My expertise includes basketball -> football, and soccer.";
    let mut delegate = Delegate::new();
    delegate.build_network(input)?;
    println!("{:?}", delegate.connections);

    // Get connections for a specific word
    if let Some(connections) = delegate.get_connections("basketball") {
        println!("Connections for 'basketball': {:?}", connections);
    }

    // Get attributes for a specific attribute name
    if let Some(interests) = delegate.get_attributes("interests") {
        println!("Interests: {:?}", interests.values);
    }

    Ok(())
}