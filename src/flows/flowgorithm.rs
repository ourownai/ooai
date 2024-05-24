use crate::blocks::{Block, InputBlock, DecisionBlock, GoToBlock, ConditionalBlock, DisplayBlock, RandomBlock, InteractiveBlock, ExternalDataBlock};
use crate::flows::{FlowDefinition, Binder};
use crate::bindings::spacy_bindings::{SpacyModule, Doc, EntityLabel};
use crate::providers::anthropic::AnthropicProvider;
use crate::flows::logic::scheduling_logic::SchedulingLogic;
use crate::flows::sample_flow::SampleFlow;
use crate::flows::blocks::{Block, InputBlock, DecisionBlock, GoToBlock, ConditionalBlock, DisplayBlock, RandomBlock, InteractiveBlock, ExternalDataBlock};
use crate::flows::block_library::BlockLibrary;

use serde_json::Value;
use std::collections::HashMap;
use rand::Rng;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub struct Flowgorithm {
    block_library: BlockLibrary,
    anthropic_provider: AnthropicProvider,
    block_templates: BlockTemplates,
}

impl Flowgorithm {
    pub fn new() -> Self {
        let block_library = BlockLibrary::new();
        let anthropic_provider = AnthropicProvider::new();
        let block_templates = BlockTemplates::new();

        Flowgorithm {
            block_library,
            anthropic_provider,
            block_templates,
        }
    }

    pub async fn process_user_instruction(&mut self, instruction: &str) -> Result<(), String> {
        // Perform NLU using spaCy
        let doc = self.perform_nlu(instruction).await?;

        // Extract entities and intents from the NLU result
        let entities = self.extract_entities(&doc)?;
        let intents = self.extract_intents(&doc)?;

        // Generate logic based on the extracted entities and intents
        let logic = self.generate_logic(&entities, &intents)?;

        // Generate a flow based on the generated logic
        let flow = self.generate_flow(&logic)?;

        // Save the generated flow
        self.save_flow(&flow)?;

        Ok(())
    }

    async fn perform_nlu(&self, text: &str) -> Result<Doc, String> {
        let spacy = SpacyModule::new();
        let doc = spacy.process(text).await?;
        Ok(doc)
    }

    fn extract_entities(&self, doc: &Doc) -> Result<Vec<EntityLabel>, String> {
        let mut entities = Vec::new();
        for ent in doc.ents() {
            let label = ent.label().to_string();
            let start = ent.start();
            let end = ent.end();
            let text = ent.text().to_string();
            let entity = EntityLabel {
                label,
                start,
                end,
                text,
            };
            entities.push(entity);
        }
        Ok(entities)
    }
    
    fn extract_intents(&self, doc: &Doc) -> Result<Vec<String>, String> {
        let mut intents = Vec::new();
        for sent in doc.sents() {
            let intent = self.classify_intent(sent)?;
            if let Some(intent) = intent {
                intents.push(intent);
            }
        }
        Ok(intents)
    }
    
    fn classify_intent(&self, sent: Sent) -> Result<Option<String>, String> {
        let intent_classifier = self.load_intent_classifier()?;
        let intent = intent_classifier.predict(sent.text())?;
        Ok(intent)
    }
    
    fn load_intent_classifier(&self) -> Result<IntentClassifier, String> {
        // Load the intent classifier model
        // ...
    }
    
    fn generate_logic(&self, entities: &[EntityLabel], intents: &[String]) -> Result<SchedulingLogic, String> {
        let mut logic = SchedulingLogic::default();
        
        // Analyze entities and intents to generate logic
        for intent in intents {
            match intent.as_str() {
                "create_task" => {
                    let task_name = self.extract_task_name(entities)?;
                    let task_duration = self.extract_task_duration(entities)?;
                    let task = Task {
                        name: task_name,
                        duration: task_duration,
                    };
                    logic.add_task(task);
                }
                "assign_task" => {
                    let task_name = self.extract_task_name(entities)?;
                    let assignee = self.extract_assignee(entities)?;
                    logic.assign_task(task_name, assignee);
                }
                "set_deadline" => {
                    let task_name = self.extract_task_name(entities)?;
                    let deadline = self.extract_deadline(entities)?;
                    logic.set_deadline(task_name, deadline);
                }
                // Handle other intents
                _ => {}
            }
        }
        
        Ok(logic)
    }
    
    fn extract_task_name(&self, entities: &[EntityLabel]) -> Result<String, String> {
        for entity in entities {
            if entity.label == "TASK" {
                return Ok(entity.text.clone());
            }
        }
        Err("Task name not found in entities".to_string())
    }
    
    fn extract_task_duration(&self, entities: &[EntityLabel]) -> Result<Duration, String> {
        for entity in entities {
            if entity.label == "DURATION" {
                let duration_str = entity.text.clone();
                let duration = self.parse_duration(&duration_str)?;
                return Ok(duration);
            }
        }
        Err("Task duration not found in entities".to_string())
    }
    
    fn parse_duration(&self, duration_str: &str) -> Result<Duration, String> {
        let parts: Vec<&str> = duration_str.split_whitespace().collect();
        if parts.len() != 2 {
            return Err("Invalid duration format".to_string());
        }
        let value: i64 = parts[0].parse().map_err(|_| "Invalid duration value".to_string())?;
        let unit = parts[1].to_lowercase();
        match unit.as_str() {
            "min" | "mins" | "minute" | "minutes" => Ok(Duration::minutes(value)),
            "hr" | "hrs" | "hour" | "hours" => Ok(Duration::hours(value)),
            "day" | "days" => Ok(Duration::days(value)),
            "week" | "weeks" => Ok(Duration::weeks(value)),
            _ => Err("Invalid duration unit".to_string()),
        }
    }
    
    fn extract_assignee(&self, entities: &[EntityLabel]) -> Result<String, String> {
        for entity in entities {
            if entity.label == "PERSON" {
                return Ok(entity.text.clone());
            }
        }
        Err("Assignee not found in entities".to_string())
    }
    
    fn extract_deadline(&self, entities: &[EntityLabel]) -> Result<DateTime, String> {
        for entity in entities {
            if entity.label == "DATE" {
                let date_str = entity.text.clone();
                let date = self.parse_date(&date_str)?;
                return Ok(date);
            }
        }
        Err("Deadline not found in entities".to_string())
    }
    
    fn parse_date(&self, date_str: &str) -> Result<DateTime, String> {
        let formats = ["%Y-%m-%d", "%m/%d/%Y", "%d/%m/%Y", "%B %d, %Y"];
        for format in &formats {
            if let Ok(date) = DateTime::parse_from_str(date_str, format) {
                return Ok(date);
            }
        }
        Err("Invalid date format".to_string())
    }
    
    fn generate_flow(&self, logic: &SchedulingLogic) -> Result<SampleFlow, String> {
        let mut flow = SampleFlow::default();
        
        // Generate flow based on the scheduling logic
        for task in logic.tasks() {
            let block_id = self.generate_block_id();
            let block_type = "TaskBlock".to_string();
            let properties = HashMap::from([
                ("name".to_string(), Value::String(task.name.clone())),
                ("duration".to_string(), Value::String(task.duration.to_string())),
            ]);
            let block = self.create_block(&block_id, &block_type, properties)?;
            flow.add_block(block);
        }
        
        for (task_name, assignee) in logic.assignments() {
            let block_id = self.generate_block_id();
            let block_type = "AssignmentBlock".to_string();
            let properties = HashMap::from([
                ("task".to_string(), Value::String(task_name.clone())),
                ("assignee".to_string(), Value::String(assignee.clone())),
            ]);
            let block = self.create_block(&block_id, &block_type, properties)?;
            flow.add_block(block);
        }
        
        for (task_name, deadline) in logic.deadlines() {
            let block_id = self.generate_block_id();
            let block_type = "DeadlineBlock".to_string();
            let properties = HashMap::from([
                ("task".to_string(), Value::String(task_name.clone())),
                ("deadline".to_string(), Value::String(deadline.to_string())),
            ]);
            let block = self.create_block(&block_id, &block_type, properties)?;
            flow.add_block(block);
        }
        
        Ok(flow)
    }
    
    fn generate_block_id(&self) -> String {
        // Generate a unique block ID
        // ...
    }
    
    fn save_flow(&mut self, flow: &SampleFlow) -> Result<(), String> {
        let flow_json = serde_json::to_string(flow).map_err(|e| e.to_string())?;
        let file_path = "path/to/save/flow.json";
        std::fs::write(file_path, flow_json).map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn generate_block(&mut self, description: &str) -> Result<Box<dyn Block>, String> {
        // Use the AnthropicProvider to generate a block based on the description
        let block_json = self.anthropic_provider.generate_block(description).await?;
        let block = self.create_block_from_json(&block_json)?;
        Ok(block)
    }

    fn pull_from_block_library(&mut self, block_id: &str) -> Result<Box<dyn Block>, String> {
        let block_hash = self.calculate_hash(block_id);
        self.block_library.get_block(block_hash)
    }

    fn push_to_block_library(&mut self, block: Box<dyn Block>) -> Result<(), String> {
        let block_hash = self.calculate_hash(&block.get_id());
        self.block_library.add_block(block_hash, block)
    }

    fn calculate_hash<T: Hash>(&self, t: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        t.hash(&mut hasher);
        hasher.finish()
    }

    fn create_block_from_json(&self, block_json: &str) -> Result<Box<dyn Block>, String> {
        let block_data: Value = serde_json::from_str(block_json).map_err(|e| e.to_string())?;
        let block_type = block_data["type"].as_str().unwrap();
        let block = self.create_block(block_type, &block_data)?;
        Ok(block)
    }

    pub fn generate_flow(&self, flow_template: &Value) -> Result<FlowDefinition, String> {
        let flow_name = flow_template["name"].as_str().unwrap().to_string();
        let start_block_id = flow_template["start_block_id"].as_str().unwrap().to_string();
        let blocks_data = flow_template["blocks"].as_array().unwrap();

        let mut blocks = Vec::new();
        for block_data in blocks_data {
            let block_type = block_data["type"].as_str().unwrap();
            let block = self.create_block(block_type, block_data)?;
            blocks.push(block);
        }

        Ok(FlowDefinition {
            name: flow_name,
            start_block_id,
            blocks,
        })
    }

    fn create_block(&self, block_type: &str, block_data: &Value) -> Result<Box<dyn Block>, String> {
        let block_id = block_data["id"].as_str().unwrap().to_string();
        let properties = block_data["properties"].as_object().unwrap().clone();

        let block = match block_type {
            "InputBlock" => self.create_input_block(&block_id, properties),
            "DecisionBlock" => self.create_decision_block(&block_id, properties, block_data),
            "GoToBlock" => self.create_goto_block(&block_id, properties),
            "ConditionalBlock" => self.create_conditional_block(&block_id, properties),
            "DisplayBlock" => self.create_display_block(&block_id, properties),
            "RandomBlock" => self.create_random_block(&block_id, properties),
            "InteractiveBlock" => self.create_interactive_block(&block_id, properties),
            "ExternalDataBlock" => self.create_external_data_block(&block_id, properties),
            _ => {
                let block_template = self.block_templates.get_template(block_type)?;
                self.create_custom_block(&block_id, properties, block_template)
            }
        };

        Ok(block)
    }

    fn create_input_block(&self, block_id: &str, properties: HashMap<String, Value>) -> Box<dyn Block> {
        let mut input_block = InputBlock::default();
        input_block.id = block_id.to_string();
        input_block.properties = properties;
        Box::new(input_block)
    }

    fn create_decision_block(&self, block_id: &str, properties: HashMap<String, Value>, block_data: &Value) -> Box<dyn Block> {
        let mut decision_block = DecisionBlock::default();
        decision_block.id = block_id.to_string();
        decision_block.properties = properties;
        decision_block.binder = self.create_binder(block_data);
        decision_block.weights = self.create_weights(block_data);
        Box::new(decision_block)
    }

    fn create_goto_block(&self, block_id: &str, properties: HashMap<String, Value>) -> Box<dyn Block> {
        let mut goto_block = GoToBlock::default();
        goto_block.id = block_id.to_string();
        goto_block.properties = properties;
        Box::new(goto_block)
    }

    fn create_conditional_block(&self, block_id: &str, properties: HashMap<String, Value>) -> Box<dyn Block> {
        let mut conditional_block = ConditionalBlock::default();
        conditional_block.id = block_id.to_string();
        conditional_block.properties = properties;
        Box::new(conditional_block)
    }

    fn create_display_block(&self, block_id: &str, properties: HashMap<String, Value>) -> Box<dyn Block> {
        let mut display_block = DisplayBlock::default();
        display_block.id = block_id.to_string();
        display_block.properties = properties;
        Box::new(display_block)
    }

    fn create_random_block(&self, block_id: &str, properties: HashMap<String, Value>) -> Box<dyn Block> {
        let mut random_block = RandomBlock::default();
        random_block.id = block_id.to_string();
        random_block.properties = properties;
        Box::new(random_block)
    }

    fn create_interactive_block(&self, block_id: &str, properties: HashMap<String, Value>) -> Box<dyn Block> {
        let mut interactive_block = InteractiveBlock::default();
        interactive_block.id = block_id.to_string();
        interactive_block.properties = properties;
        Box::new(interactive_block)
    }

    fn create_external_data_block(&self, block_id: &str, properties: HashMap<String, Value>) -> Box<dyn Block> {
        let mut external_data_block = ExternalDataBlock::default();
        external_data_block.id = block_id.to_string();
        external_data_block.properties = properties;
        Box::new(external_data_block)
    }

    fn create_custom_block(&self, block_id: &str, properties: HashMap<String, Value>, block_template: &Value) -> Box<dyn Block> {
        let block_type = block_template["type"].as_str().unwrap();
        let mut custom_block = CustomBlock::default();
        custom_block.id = block_id.to_string();
        custom_block.block_type = block_type.to_string();
        custom_block.properties = properties;
    
        // Set the API integration if provided in the block template
        if let Some(api_integration) = block_template["api_integration"].as_object() {
            let endpoint = api_integration["endpoint"].as_str().unwrap().to_string();
            let request_format = serde_json::from_value(api_integration["request_format"].clone()).unwrap();
            let response_format = serde_json::from_value(api_integration["response_format"].clone()).unwrap();
            let authentication = serde_json::from_value(api_integration["authentication"].clone()).unwrap();
    
            custom_block.api_integration = Some(ApiIntegration {
                endpoint,
                request_format,
                response_format,
                authentication,
            });
        }
    
        // Set the parameters schema if provided in the block template
        if let Some(parameters_schema) = block_template["parameters_schema"].as_object() {
            custom_block.parameters_schema = Some(serde_json::to_value(parameters_schema).unwrap());
        }
    
        // Set the methods if provided in the block template
        if let Some(methods) = block_template["methods"].as_object() {
            let mut custom_methods = HashMap::new();
            for (method_name, method_data) in methods {
                let args = method_data["args"].as_array().unwrap().iter().map(|arg| arg.as_str().unwrap().to_string()).collect();
                let kwargs = method_data["kwargs"].as_object().unwrap().clone();
                custom_methods.insert(method_name.to_string(), CustomMethod { args, kwargs });
            }
            custom_block.methods = custom_methods;
        }
    
        Box::new(custom_block)
    }

    fn create_binder(&self, block_data: &Value) -> Option<Binder> {
        let connections = block_data["connections"].as_object().unwrap();
        let mut binder = Binder::new();
        for (from_block_id, to_block_id) in connections {
            binder.add_connection(from_block_id.to_string(), to_block_id.as_str().unwrap().to_string());
        }
        Some(binder)
    }

    fn create_weights(&self, block_data: &Value) -> Option<HashMap<String, f64>> {
        let weights_data = block_data["weights"].as_object().unwrap();
        let mut weights = HashMap::new();
        for (block_id, weight) in weights_data {
            weights.insert(block_id.to_string(), weight.as_f64().unwrap());
        }
        Some(weights)
    }
}