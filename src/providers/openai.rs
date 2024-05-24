use crate::provider_types::ai::{AIProviderManager, GenerationRequest, GenerationResponse, InferenceRequest, InferenceResponse};
use crate::messaging::message::Message;
use crate::provider_types::ai::{AIProviderTrait, ProviderInfo};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;


#[derive(Serialize, Deserialize)]
struct OpenAIGenerationRequest {
    prompt: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    n: Option<u32>,
}


#[derive(Serialize, Deserialize)]
struct OpenAIGenerationResponse {
    choices: Vec<OpenAIGenerationChoice>,
}

#[derive(Serialize, Deserialize)]
struct OpenAIGenerationChoice {
    text: String,
}

struct OpenAIProvider {
    api_key: String,
    client: Client,
}


impl OpenAIProvider {
    fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl AIProviderTrait for OpenAIProvider {
    async fn run_inference(&self, _request: InferenceRequest) -> Result<InferenceResponse, reqwest::Error> {
        unimplemented!("Inference is not supported by OpenAI provider")
    }

    async fn get_provider_info(&self) -> Result<ProviderInfo, reqwest::Error> {
        Ok(ProviderInfo {
            name: "OpenAI".to_string(),
            description: "OpenAI provider using the GPT-3 API".to_string(),
            capabilities: vec!["text-generation".to_string()],
        })
    }    

    async fn run_generation(&self, request: GenerationRequest) -> Result<GenerationResponse, reqwest::Error> {
        let openai_request = OpenAIGenerationRequest {
            prompt: request.message.content,
            max_tokens: request.max_length,
            temperature: request.temperature,
            n: request.n_best,
        };

        let response = self.client
            .post("https://api.openai.com/v1/engines/davinci-codex/completions")
            .bearer_auth(&self.api_key)
            .json(&openai_request)
            .send()
            .await?
            .json::<OpenAIGenerationResponse>()
            .await?;

        let mut message = request.message;
        if let Some(choice) = response.choices.first() {
            message.content = choice.text.clone();
        }

        Ok(GenerationResponse {
            message,
            model_used: Some("openai-davinci-codex".to_string()),
        })
    }
}

#[tokio::main]
async fn main() {
    // Load the OpenAI API key from an environment variable
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

    // Create an instance of the OpenAI provider
    let openai_provider = OpenAIProvider::new(&api_key);

    // Create an instance of the AI provider manager
    let mut ai_manager = AIProviderManager::new(HashMap::new(), "default");

    // Add the OpenAI provider to the AI provider manager
    ai_manager.add_provider("openai", openai_provider);

    // Example usage: Generate text using the OpenAI provider
    let input_message = Message {
        id: Uuid::new_v4(),
        channel_id: "".to_string(),
        sender: "".to_string(),
        recipient: "".to_string(),
        timestamp: chrono::Utc::now(),
        edited_at: None,
        hash: "".to_string(),
        feedback_weights: Default::default(),
        content: "Once upon a time".to_string(),
        metadata: Default::default(),
        text: "Once upon a time".to_string(),
        intent: None,
        payment: None,
        nonce: None,
        name: None,
        data: None,
        header: None,
        body: None,
        contexts: vec![],
        values: vec![],
        entity_graph: None,
    };
    
    

    let generation_request = GenerationRequest {
        message: input_message,
        max_length: Some(100),
        temperature: Some(0.7),
        n_best: Some(1),
    };

    match ai_manager.run_generation(generation_request).await {
        Ok(response) => {
            println!("Generated text: {}", response.message.content);
        }
        Err(error) => {
            eprintln!("Error: {}", error);
        }
    }
}