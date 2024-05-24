//! # Flows
//!
//! This module provides a framework for defining and executing workflows or flows in a Rust application.
//!
//! ## Overview
//!
//! A flow represents a series of steps or tasks that are executed in a specific order to achieve a desired outcome.
//! The `flows` module allows you to define these steps, specify their dependencies, and execute them efficiently.
//!
//! ## Key Components
//!
//! The main components of the `flows` module are:
//!
//! - `FlowDefinition`: A struct that represents the definition of a flow, including its name, start block ID, and blocks.
//! - `Block`: A struct that represents a single block in a flow, containing its ID, type, properties, binder, and weights.
//! - `Binder`: A struct that manages the connections between blocks in a flow.
//! - `FlowEngine`: A struct responsible for executing flows based on their definitions and a graph of block connections.
//!
//! ## Defining a Flow
//!
//! To define a flow, you need to create a `FlowDefinition` struct that includes the following:
//!
//! - `name`: The name of the flow.
//! - `start_block_id`: The ID of the starting block in the flow.
//! - `blocks`: A vector of `Block` structs representing the blocks in the flow.
//!
//! Each `Block` struct contains the following fields:
//!
//! - `id`: The unique identifier of the block.
//! - `block_type`: The type of the block (e.g., "DecisionBlock", "GoToBlock").
//! - `properties`: A map of key-value pairs representing the properties of the block.
//! - `binder`: An optional `Binder` struct that manages the connections between blocks.
//! - `weights`: An optional map of weights for each outgoing connection from the block.
//! - `graph_weights`: An optional map of weights calculated based on the graph of block connections.
//!
//! ## Executing a Flow
//!
//! To execute a flow, you need to create an instance of the `FlowEngine` struct, providing the flow definitions and a graph of block connections.
//!
//! The `execute_flow` method of the `FlowEngine` takes the name of the flow to execute and a map of input data. It returns a `Result` indicating the success or failure of the flow execution.
//!
//! During the execution of a flow, the `FlowEngine` iterates through the blocks, starting from the `start_block_id` specified in the `FlowDefinition`. Each block is processed based on its type, and the execution continues to the next block based on the block's binder, weights, or graph weights.
//!
//! ## Block Processing
//!
//! The `process` method of the `Block` struct is responsible for processing a block based on its type. It takes the `FlowEngine` instance and a mutable reference to the state (a map of key-value pairs) and returns a `BlockResult` enum indicating the next action to take.
//!
//! The supported block types and their processing logic are as follows:
//!
//! - `DecisionBlock`: Evaluates the conditions specified in the block's properties and determines the next block to move to based on the matching condition.
//! - `GoToBlock`: Moves the execution to the block specified by the `destination_block_id` property.
//!
//! ## Error Handling
//!
//! The `flows` module uses `Result` types to handle errors during flow execution. If an error occurs, it is propagated to the caller of the `execute_flow` method.
//!
//! ## Utility Functions
//!
//! The `flows` module provides utility functions for loading flow definitions and graphs from JSON files, as well as creating blocks based on a configuration.
//!
//! - `load_flow_definitions`: Loads flow definitions from a JSON file and returns a map of flow names to `FlowDefinition` structs.
//! - `load_graph`: Loads a graph of block connections from a JSON file and returns a map of block IDs to vectors of connected block IDs.
//! - create_block: Creates a Block struct based on the provided configuration, which includes the block ID, type, properties, binder, and weights.


use rand::Rng;
use crate::blocks::{Block, BlockResult, InputBlock, DecisionBlock, GoToBlock, ConditionalBlock, DisplayBlock, RandomBlock, InteractiveBlock, ExternalDataBlock};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Serialize)]
pub struct FlowDefinition {
    pub name: String,
    pub start_block_id: String,
    pub blocks: Vec<Block>,
}

#[derive(Deserialize, Serialize)]
pub struct Binder {
    connections: HashMap<String, String>,
}

impl Binder {
    fn new() -> Self {
        Binder {
            connections: HashMap::new(),
        }
    }

    fn add_connection(&mut self, from_block_id: String, to_block_id: String) {
        self.connections.insert(from_block_id, to_block_id);
    }

    fn get_next_block_id(&self, current_block_id: &str) -> Option<&String> {
        self.connections.get(current_block_id)
    }
}

pub struct FlowEngine {
    flow_definitions: HashMap<String, FlowDefinition>,
    graph: HashMap<String, Vec<String>>,
}

impl FlowEngine {
    pub fn new(flow_definitions: HashMap<String, FlowDefinition>, graph: HashMap<String, Vec<String>>) -> Self {
        FlowEngine {
            flow_definitions,
            graph,
        }
    }

    pub async fn execute_flow(&self, flow_name: &str, input_data: HashMap<String, serde_json::Value>) -> Result<(), String> {
        let flow_definition = self.flow_definitions.get(flow_name).ok_or_else(|| format!("Flow not found: {}", flow_name))?;
        let mut state = input_data;
        let mut current_block_id = flow_definition.start_block_id.clone();

        loop {
            let block = get_block_by_id(&flow_definition, &current_block_id)?;
            let result = self.process_block(block, &mut state).await?;

            match result {
                BlockResult::Move(connection) => {
                    if let Some(binder) = &block.binder() {
                        if let Some(next_block_id) = binder.get_next_block_id(&block.id()) {
                            current_block_id = next_block_id.clone();
                        } else {
                            return Ok(());
                        }
                    } else if let Some(weights) = &block.weights() {
                        let mut rng = rand::thread_rng();
                        let mut total_weight = 0.0;
                        for weight in weights.values() {
                            total_weight += weight;
                        }
                        let mut random_weight = rng.gen_range(0.0..total_weight);
                        for (block_id, weight) in weights {
                            random_weight -= weight;
                            if random_weight <= 0.0 {
                                current_block_id = block_id.clone();
                                break;
                            }
                        }
                    } else {
                        return Ok(());
                    }
                }
                BlockResult::Reject(reason) => {
                    return Err(reason);
                }
                BlockResult::Terminate => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn process_block(&self, block: &Block, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        match block {
            Block::InputBlock(input_block) => self.process_input_block(input_block, state).await,
            Block::DecisionBlock(decision_block) => self.process_decision_block(decision_block, state).await,
            Block::GoToBlock(goto_block) => self.process_goto_block(goto_block, state).await,
            Block::ConditionalBlock(conditional_block) => self.process_conditional_block(conditional_block, state).await,
            Block::DisplayBlock(display_block) => self.process_display_block(display_block, state).await,
            Block::RandomBlock(random_block) => self.process_random_block(random_block, state).await,
            Block::InteractiveBlock(interactive_block) => self.process_interactive_block(interactive_block, state).await,
            Block::ExternalDataBlock(external_data_block) => self.process_external_data_block(external_data_block, state).await,
        }
    }

    async fn process_input_block(&self, input_block: &InputBlock, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        input_block.on_process(state, None).await
    }

    async fn process_decision_block(&self, decision_block: &DecisionBlock, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        decision_block.process(self, state).await
    }

    async fn process_goto_block(&self, goto_block: &GoToBlock, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        goto_block.process(self, state).await
    }

    async fn process_conditional_block(&self, conditional_block: &ConditionalBlock, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        conditional_block.process(self, state).await
    }

    async fn process_display_block(&self, display_block: &DisplayBlock, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        display_block.process(self, state).await
    }

    async fn process_random_block(&self, random_block: &RandomBlock, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        random_block.process(self, state).await
    }

    async fn process_interactive_block(&self, interactive_block: &InteractiveBlock, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        interactive_block.process(self, state).await
    }

    async fn process_external_data_block(&self, external_data_block: &ExternalDataBlock, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        external_data_block.process(self, state).await
    }

    fn resolve_variables(&self, template: &str, state: &HashMap<String, serde_json::Value>) -> String {
        let mut resolved = template.to_string();
        for (key, value) in state {
            let placeholder = format!("{{{{{}}}}}", key);
            resolved = resolved.replace(&placeholder, value.as_str().unwrap_or(""));
        }
        resolved
    }

    fn calculate_block_weights(&mut self, flow_definition: &mut FlowDefinition) {
        for block in &mut flow_definition.blocks {
            self.calculate_graph_weights(block);
        }
    }

    fn calculate_graph_weights(&mut self, block: &mut Block) {
        match block {
            Block::InputBlock(input_block) => input_block.calculate_graph_weights(&self.graph),
            Block::DecisionBlock(decision_block) => decision_block.calculate_graph_weights(&self.graph),
            Block::GoToBlock(goto_block) => goto_block.calculate_graph_weights(&self.graph),
            Block::ConditionalBlock(conditional_block) => conditional_block.calculate_graph_weights(&self.graph),
            Block::DisplayBlock(display_block) => display_block.calculate_graph_weights(&self.graph),
            Block::RandomBlock(random_block) => random_block.calculate_graph_weights(&self.graph),
            Block::InteractiveBlock(interactive_block) => interactive_block.calculate_graph_weights(&self.graph),
            Block::ExternalDataBlock(external_data_block) => external_data_block.calculate_graph_weights(&self.graph),
        }
    }
}

fn get_block_by_id(flow_definition: &FlowDefinition, block_id: &str) -> Result<&Block, String> {
    flow_definition.blocks.iter().find(|block| block.id() == block_id).ok_or_else(|| format!("Block not found: {}", block_id))
}

fn load_flow_definitions(file_path: &str) -> Result<HashMap<String, FlowDefinition>, String> {
    let file_contents = fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;
    let flow_definitions: HashMap<String, FlowDefinition> = serde_json::from_str(&file_contents).map_err(|e| format!("Failed to parse JSON: {}", e))?;
    Ok(flow_definitions)
}

fn main() {
    let flow_definitions = load_flow_definitions("flow_definitions.json").unwrap();
    let graph = load_graph("graph.json").unwrap();
    let mut engine = FlowEngine::new(flow_definitions, graph);

    for flow_definition in engine.flow_definitions.values_mut() {
        engine.calculate_block_weights(flow_definition);
    }

    let input_data = HashMap::new();
    let result = tokio::runtime::Runtime::new().unwrap().block_on(engine.execute_flow("example_flow", input_data));

    match result {
        Ok(()) => println!("Flow executed successfully"),
        Err(e) => println!("Flow execution failed: {}", e),
    }
}

fn load_graph(file_path: &str) -> Result<HashMap<String, Vec<String>>, String> {
    let file_contents = fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;
    let graph: HashMap<String, Vec<String>> = serde_json::from_str(&file_contents).map_err(|e| format!("Failed to parse JSON: {}", e))?;
    Ok(graph)
}