use std::collections::HashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

use crate::provider_types::ai::{AIProviderManager, GenerationRequest, GenerationResponse, InferenceRequest, InferenceResponse};
use crate::messaging::message::Message;

#[derive(Serialize, Deserialize)]
struct AnthropicGenerationRequest {
    prompt: String,
    max_tokens_to_sample: Option<u32>,
    temperature: Option<f32>,
    top_k: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct AnthropicGenerationResponse {
    output: String,
}

struct AnthropicProvider {
    api_key: String,
    client: Client,
}

impl AnthropicProvider {
    fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    async fn run_generation(&self, request: GenerationRequest) -> Result<GenerationResponse, reqwest::Error> {
        let anthropic_request = AnthropicGenerationRequest {
            prompt: request.message.content,
            max_tokens_to_sample: request.max_length,
            temperature: request.temperature,
            top_k: request.n_best,
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/complete")
            .bearer_auth(&self.api_key)
            .json(&anthropic_request)
            .send()
            .await?
            .json::<AnthropicGenerationResponse>()
            .await?;

        let mut message = request.message;
        message.content = response.output;

        Ok(GenerationResponse {
            message,
            model_used: Some("anthropic".to_string()),
        })
    }
}

#[tokio::main]
async fn main() {
    // Load the Anthropic API key from an environment variable
    let api_key = env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");

    // Create an instance of the Anthropic provider
    let anthropic_provider = AnthropicProvider::new(&api_key);

    // Create an instance of the AI provider manager
    let mut ai_manager = AIProviderManager::new(HashMap::new(), "default");

    // Add the Anthropic provider to the AI provider manager
    ai_manager.add_provider("anthropic", anthropic_provider);

    // Example usage: Generate text using the Anthropic provider
    let input_message = Message {
        id: Default::default(),
        channel_id: Default::default(),
        sender: Default::default(),
        recipient: Default::default(),
        timestamp: Default::default(),
        edited_at: Default::default(),
        hash: Default::default(),
        feedback_weights: Default::default(),
        text: "Once upon a time".to_string(),
        content: Default::default(),
        metadata: Default::default(),
        intent: Default::default(),
        payment: Default::default(),
        nonce: Default::default(),
        name: Default::default(),
        data: Default::default(),
        header: Default::default(),
        body: Default::default(),
        contexts: Default::default(),
        values: Default::default(),
        entity_graph: Default::default(),
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