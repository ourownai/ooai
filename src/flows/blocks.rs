use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::flows::{FlowEngine, BlockResult, Binder};

pub trait Block {
    fn id(&self) -> &str;
    fn process(&self, engine: &FlowEngine, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String>;
    fn binder(&self) -> Option<&Binder>;
    fn weights(&self) -> Option<&HashMap<String, f64>>;
    fn calculate_graph_weights(&mut self, graph: &HashMap<String, Vec<String>>);
}

#[derive(Deserialize, Serialize)]
pub struct InputBlock {
    pub id: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub api_integration: Option<ApiIntegration>,
    pub parameters_schema: Option<serde_json::Value>,
}

impl Block for InputBlock {
    fn id(&self) -> &str {
        &self.id
    }

    fn process(&self, engine: &FlowEngine, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        self.on_process(state, None)
    }

    fn binder(&self) -> Option<&Binder> {
        None
    }

    fn weights(&self) -> Option<&HashMap<String, f64>> {
        None
    }

    fn calculate_graph_weights(&mut self, graph: &HashMap<String, Vec<String>>) {
        // No graph weights for InputBlock
    }
}

impl InputBlock {
    pub fn on_process(&self, state: &mut HashMap<String, serde_json::Value>, input: Option<serde_json::Value>) -> Result<BlockResult, String> {
        if !self.get_property("required").unwrap_or(&serde_json::Value::Bool(true)).as_bool().unwrap() && input.is_none() {
            self.save(state, None);
            Ok(BlockResult::Move("Next".to_string()))
        } else {
            self.process_input(state, input)
        }
    }

    fn process_input(&self, state: &mut HashMap<String, serde_json::Value>, input: Option<serde_json::Value>) -> Result<BlockResult, String> {
        if let Some(value) = input {
            // Validate the input against the parameters_schema if provided
            if let Some(schema) = &self.parameters_schema {
                if let Err(e) = serde_json::from_value::<serde_json::Value>(value.clone()) {
                    return Err(format!("Invalid input: {}", e));
                }
            }
            self.save(state, Some(value));
            Ok(BlockResult::Move("Next".to_string()))
        } else {
            Err("Input is required".to_string())
        }
    }

    fn save(&self, state: &mut HashMap<String, serde_json::Value>, value: Option<serde_json::Value>) {
        let key = self.get_property("key").unwrap().as_str().unwrap();
        state.insert(key.to_string(), value.unwrap_or(serde_json::Value::Null));
    }

    fn load(&self, state: &HashMap<String, serde_json::Value>) -> Option<serde_json::Value> {
        let key = self.get_property("key").unwrap().as_str().unwrap();
        state.get(key).cloned()
    }

    fn load_template(&self) {
        let key = self.get_property("key").unwrap().as_str().unwrap();
        let prompt = self.get_property("prompt").unwrap().as_str().unwrap();
        let required = self.get_property("required").unwrap().as_bool().unwrap();
        // Add template properties based on the schema
        // ...
    }

    fn on_search(&self, query: &str) -> Vec<serde_json::Value> {
        let required = self.get_property("required").unwrap_or(&serde_json::Value::Bool(true)).as_bool().unwrap();
        if !required {
            let mut resources = self.on_search_default(query);
            resources.push(serde_json::json!({
                "type": "SearchNode",
                "value": "SKIP",
            }));
            resources
        } else {
            self.on_search_default(query)
        }
    }

    fn on_search_default(&self, query: &str) -> Vec<serde_json::Value> {
        // Perform default search logic based on the query
        let key = self.get_property("key").unwrap().as_str().unwrap();
        let prompt = self.get_property("prompt").unwrap().as_str().unwrap();
        let mut results = Vec::new();
        if key.contains(query) || prompt.contains(query) {
            results.push(serde_json::json!({
                "type": "SearchNode",
                "value": self.id.clone(),
            }));
        }
        results
    }

    fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }
}

#[derive(Deserialize, Serialize)]
pub struct DecisionBlock {
    pub id: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub binder: Option<Binder>,
    pub weights: Option<HashMap<String, f64>>,
    pub graph_weights: Option<HashMap<String, f64>>,
}

impl Block for DecisionBlock {
    fn id(&self) -> &str {
        &self.id
    }

    fn process(&self, engine: &FlowEngine, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        let options = self.get_property("connections").unwrap().as_array().unwrap();
        for option in options {
            let value = option.get("value").unwrap().as_str().unwrap();
            let condition = option.get("condition").unwrap().as_str().unwrap();
            if condition.is_empty() || self.evaluate_condition(condition, state) {
                return Ok(BlockResult::Move(value.to_string()));
            }
        }
        Ok(BlockResult::Reject("No matching condition".to_string()))
    }

    fn binder(&self) -> Option<&Binder> {
        self.binder.as_ref()
    }

    fn weights(&self) -> Option<&HashMap<String, f64>> {
        self.weights.as_ref()
    }

    fn calculate_graph_weights(&mut self, graph: &HashMap<String, Vec<String>>) {
        if let Some(connections) = self.properties.get("connections") {
            let mut graph_weights = HashMap::new();
            for connection in connections.as_array().unwrap() {
                let target_block_id = connection.get("value").unwrap().as_str().unwrap();
                let weight = self.calculate_connection_weight(graph, target_block_id);
                graph_weights.insert(target_block_id.to_string(), weight);
            }
            self.graph_weights = Some(graph_weights);
        }
    }
}

impl DecisionBlock {
    fn evaluate_condition(&self, condition: &str, state: &HashMap<String, serde_json::Value>) -> bool {
        // Evaluate the condition based on the state
        let mut context = rhai::Map::new();
        for (key, value) in state {
            context.insert(key.clone(), value.clone());
        }
        let engine = rhai::Engine::new();
        engine.eval_with_scope(&mut context, condition).unwrap_or(false)
    }

    fn calculate_connection_weight(&self, graph: &HashMap<String, Vec<String>>, target_block_id: &str) -> f64 {
        // Calculate the connection weight based on the graph
        let mut total_connections = 0;
        let mut target_connections = 0;
        for (_, connections) in graph {
            total_connections += connections.len();
            if connections.contains(&target_block_id.to_string()) {
                target_connections += 1;
            }
        }
        if total_connections == 0 {
            0.0
        } else {
            target_connections as f64 / total_connections as f64
        }
    }

    fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }
}

pub trait Block {
    fn id(&self) -> &str;
    fn process(&self, engine: &FlowEngine, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String>;
    fn binder(&self) -> Option<&Binder>;
    fn weights(&self) -> Option<&HashMap<String, f64>>;
    fn calculate_graph_weights(&mut self, graph: &HashMap<String, Vec<String>>);
}

// InputBlock implementation (same as before)

#[derive(Deserialize, Serialize)]
pub struct DecisionBlock {
    pub id: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub binder: Option<Binder>,
    pub weights: Option<HashMap<String, f64>>,
    pub graph_weights: Option<HashMap<String, f64>>,
}

// DecisionBlock implementation (same as before)

#[derive(Deserialize, Serialize)]
pub struct GoToBlock {
    pub id: String,
    pub properties: HashMap<String, serde_json::Value>,
}

impl Block for GoToBlock {
    fn id(&self) -> &str {
        &self.id
    }

    fn process(&self, engine: &FlowEngine, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        let destination_block_id = self.get_property("destination_block_id").unwrap().as_str().unwrap();
        Ok(BlockResult::Move(destination_block_id.to_string()))
    }

    fn binder(&self) -> Option<&Binder> {
        None
    }

    fn weights(&self) -> Option<&HashMap<String, f64>> {
        None
    }

    fn calculate_graph_weights(&mut self, graph: &HashMap<String, Vec<String>>) {
        // No graph weights for GoToBlock
    }
}

impl GoToBlock {
    fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }
}

#[derive(Deserialize, Serialize)]
pub struct ConditionalBlock {
    pub id: String,
    pub properties: HashMap<String, serde_json::Value>,
}

impl Block for ConditionalBlock {
    fn id(&self) -> &str {
        &self.id
    }

    fn process(&self, engine: &FlowEngine, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        let condition = self.get_property("condition").unwrap().as_str().unwrap();
        if self.evaluate_condition(condition, state) {
            let true_block_id = self.get_property("true_block_id").unwrap().as_str().unwrap();
            Ok(BlockResult::Move(true_block_id.to_string()))
        } else {
            let false_block_id = self.get_property("false_block_id").unwrap().as_str().unwrap();
            Ok(BlockResult::Move(false_block_id.to_string()))
        }
    }

    fn binder(&self) -> Option<&Binder> {
        None
    }

    fn weights(&self) -> Option<&HashMap<String, f64>> {
        None
    }

    fn calculate_graph_weights(&mut self, graph: &HashMap<String, Vec<String>>) {
        // No graph weights for ConditionalBlock
    }
}

impl ConditionalBlock {
    fn evaluate_condition(&self, condition: &str, state: &HashMap<String, serde_json::Value>) -> bool {
        // Evaluate the condition based on the state
        let mut context = rhai::Map::new();
        for (key, value) in state {
            context.insert(key.clone(), value.clone());
        }
        let engine = rhai::Engine::new();
        engine.eval_with_scope(&mut context, condition).unwrap_or(false)
    }

    fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }
}

#[derive(Deserialize, Serialize)]
pub struct DisplayBlock {
    pub id: String,
    pub properties: HashMap<String, serde_json::Value>,
}

impl Block for DisplayBlock {
    fn id(&self) -> &str {
        &self.id
    }

    fn process(&self, engine: &FlowEngine, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        let message = self.get_property("message").unwrap().as_str().unwrap();
        let resolved_message = engine.resolve_variables(message, state);
        println!("{}", resolved_message);
        Ok(BlockResult::Move("Next".to_string()))
    }

    fn binder(&self) -> Option<&Binder> {
        None
    }

    fn weights(&self) -> Option<&HashMap<String, f64>> {
        None
    }

    fn calculate_graph_weights(&mut self, graph: &HashMap<String, Vec<String>>) {
        // No graph weights for DisplayBlock
    }
}

impl DisplayBlock {
    fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }
}

#[derive(Deserialize, Serialize)]
pub struct RandomBlock {
    pub id: String,
    pub properties: HashMap<String, serde_json::Value>,
}

impl Block for RandomBlock {
    fn id(&self) -> &str {
        &self.id
    }

    fn process(&self, engine: &FlowEngine, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        let options = self.get_property("options").unwrap().as_array().unwrap();
        let mut rng = rand::thread_rng();
        let mut total_weight = 0.0;
        for option in options {
            let weight = option.get("weight").unwrap().as_f64().unwrap();
            total_weight += weight;
        }
        let mut random_weight = rng.gen_range(0.0..total_weight);
        for option in options {
            let block_id = option.get("block_id").unwrap().as_str().unwrap();
            let weight = option.get("weight").unwrap().as_f64().unwrap();
            random_weight -= weight;
            if random_weight <= 0.0 {
                return Ok(BlockResult::Move(block_id.to_string()));
            }
        }
        Err("No matching option found".to_string())
    }

    fn binder(&self) -> Option<&Binder> {
        None
    }

    fn weights(&self) -> Option<&HashMap<String, f64>> {
        None
    }

    fn calculate_graph_weights(&mut self, graph: &HashMap<String, Vec<String>>) {
        // No graph weights for RandomBlock
    }
}

impl RandomBlock {
    fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }
}

#[derive(Deserialize, Serialize)]
pub struct InteractiveBlock {
    pub id: String,
    pub properties: HashMap<String, serde_json::Value>,
}

impl Block for InteractiveBlock {
    fn id(&self) -> &str {
        &self.id
    }

    fn process(&self, engine: &FlowEngine, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        let question = self.get_property("question").unwrap().as_str().unwrap();
        let options = self.get_property("options").unwrap().as_array().unwrap();
        println!("{}", question);
        for (index, option) in options.iter().enumerate() {
            let text = option.get("text").unwrap().as_str().unwrap();
            println!("{}. {}", index + 1, text);
        }
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let selected_index = input.trim().parse::<usize>().unwrap() - 1;
        let selected_option = options.get(selected_index).unwrap();
        let next_block_id = selected_option.get("next_block_id").unwrap().as_str().unwrap();
        Ok(BlockResult::Move(next_block_id.to_string()))
    }

    fn binder(&self) -> Option<&Binder> {
        None
    }

    fn weights(&self) -> Option<&HashMap<String, f64>> {
        None
    }

    fn calculate_graph_weights(&mut self, graph: &HashMap<String, Vec<String>>) {
        // No graph weights for InteractiveBlock
    }
}

impl InteractiveBlock {
    fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }
}

#[derive(Deserialize, Serialize)]
pub struct ExternalDataBlock {
    pub id: String,
    pub properties: HashMap<String, serde_json::Value>,
}

impl Block for ExternalDataBlock {
    fn id(&self) -> &str {
        &self.id
    }

    fn process(&self, engine: &FlowEngine, state: &mut HashMap<String, serde_json::Value>) -> Result<BlockResult, String> {
        let api_url = self.get_property("api_url").unwrap().as_str().unwrap();
        let data_path = self.get_property("data_path").unwrap().as_str().unwrap();
        let response = reqwest::blocking::get(api_url).map_err(|e| format!("API request failed: {}", e))?;
        let json_data: serde_json::Value = response.json().map_err(|e| format!("Failed to parse JSON response: {}", e))?;
        let data = json_data.pointer(data_path).unwrap().clone();
        state.insert("external_data".to_string(), data);
        Ok(BlockResult::Move("Next".to_string()))
    }

    fn binder(&self) -> Option<&Binder> {
        None
    }

    fn weights(&self) -> Option<&HashMap<String, f64>> {
        None
    }

    fn calculate_graph_weights(&mut self, graph: &HashMap<String, Vec<String>>) {
        // No graph weights for ExternalDataBlock
    }
}

impl ExternalDataBlock {
    fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }
}

// ApiIntegration, RequestFormat, ResponseFormat, ResponseStatus, and Authentication structs (same as before)

#[derive(Deserialize, Serialize)]
pub struct ApiIntegration {
    pub endpoint: String,
    pub request_format: RequestFormat,
    pub response_format: ResponseFormat,
    pub authentication: Authentication,
}

#[derive(Deserialize, Serialize)]
pub struct RequestFormat {
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
}

#[derive(Deserialize, Serialize)]
pub struct ResponseFormat {
    pub success: ResponseStatus,
    pub error: ResponseStatus,
}

#[derive(Deserialize, Serialize)]
pub struct ResponseStatus {
    pub status_code: u16,
    pub body: serde_json::Value,
}

#[derive(Deserialize, Serialize)]
pub struct Authentication {
    #[serde(rename = "type")]
    pub auth_type: String,
    pub token: String,
}
