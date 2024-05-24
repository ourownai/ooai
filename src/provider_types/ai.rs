//! # AI Provider Module
//!
//! This module provides functionality for interacting with AI providers and managing AI-related tasks.
//!
//! ## Structures
//!
//! - `InferenceRequest`: Represents a request for running inference on a message using an AI provider.
//! - `InferenceResponse`: Represents the response from running inference on a message.
//! - `GenerationRequest`: Represents a request for generating a message using an AI provider.
//! - `GenerationResponse`: Represents the response from generating a message.
//! - `AIProvider`: Implements the `AIProviderTrait` for interacting with an AI provider's API.
//! - `ProviderInfo`: Represents information about an AI provider, including its name, description, and capabilities.
//! - `ProviderSelector`: Manages multiple AI providers and selects the appropriate provider based on criteria.
//! - `AIProviderManager`: Orchestrates the usage of AI providers for running inference and generation tasks.
//!
//! ## Traits
//!
//! - `AIProviderTrait`: Defines the interface for an AI provider, including methods for running inference, generation, and retrieving provider information.
//!
//! ## Functions
//!
//! - `AIProviderManager::new`: Creates a new instance of `AIProviderManager` with the provided API keys and default provider key.
//! - `AIProviderManager::run_inference`: Runs inference on a message using the selected AI provider based on the message classification.
//! - `AIProviderManager::run_generation`: Generates a message using the selected AI provider based on the message classification.
//!
//! ## Usage
//!
//! 1. Create an instance of `AIProviderManager` with the desired API keys and default provider key.
//! 2. Call `run_inference` or `run_generation` on the `AIProviderManager` instance, passing the appropriate message.
//! 3. The `AIProviderManager` will select the suitable AI provider based on the message classification and use it to run the requested task.
//! 4. The response from the AI provider will be returned, and the generated message will be routed using the `MessageRouter`.
//!

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::messaging::message::Message;
use crate::messaging::message_classifier::classify_message;


#[derive(Serialize, Deserialize)]
pub struct InferenceRequest {
    pub message: Message,
    pub model: Option<String>,
    pub parameters: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
pub struct InferenceResponse {
    pub message: Message,
    pub confidence: Option<f32>,
    pub model_used: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GenerationRequest {
    pub message: Message,
    pub max_length: Option<u32>,
    pub temperature: Option<f32>,
    pub n_best: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct GenerationResponse {
    pub message: Message,
    pub model_used: Option<String>,
}

#[async_trait::async_trait]
pub trait AIProviderTrait {
    async fn run_inference(&self, request: InferenceRequest) -> Result<InferenceResponse, reqwest::Error>;
    async fn run_generation(&self, request: GenerationRequest) -> Result<GenerationResponse, reqwest::Error>;
    async fn get_provider_info(&self) -> Result<ProviderInfo, reqwest::Error>;
}

struct AIProvider {
    pub api_key: String,
    pub client: Client,
    pub base_url: String,
}

#[async_trait::async_trait]
impl AIProviderTrait for AIProvider {
    async fn run_inference(&self, request: InferenceRequest) -> Result<InferenceResponse, reqwest::Error> {
        let response = self.client
            .post(&format!("{}/inference", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?
            .json::<InferenceResponse>()
            .await?;
        Ok(response)
    }

    async fn run_generation(&self, request: GenerationRequest) -> Result<GenerationResponse, reqwest::Error> {
        let response = self.client
            .post(&format!("{}/generation", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?
            .json::<GenerationResponse>()
            .await?;
        Ok(response)
    }

    async fn get_provider_info(&self) -> Result<ProviderInfo, reqwest::Error> {
        let response = self.client
            .get(&format!("{}/info", self.base_url))
            .bearer_auth(&self.api_key)
            .send()
            .await?
            .json::<ProviderInfo>()
            .await?;
        Ok(response)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
}

pub struct ProviderSelector {
    pub providers: HashMap<String, Arc<Mutex<dyn AIProviderTrait + Send + Sync>>>,
}

impl ProviderSelector {
    fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    fn add_provider(&mut self, key: &str, provider: impl AIProviderTrait + Send + Sync + 'static) {
        self.providers.insert(key.to_string(), Arc::new(Mutex::new(provider)));
    }

    async fn select_provider(
        &self,
        provider_graph: &HashMap<String, HashMap<String, f32>>,
        user_preference_graph: &HashMap<String, HashMap<String, f32>>,
        topic_graph: &HashMap<String, HashMap<String, f32>>,
        message: &Message,
    ) -> Option<Arc<Mutex<dyn AIProviderTrait + Send + Sync>>> {
        let mut scores: HashMap<String, f32> = HashMap::new();
        for (provider_key, provider) in &self.providers {
            let provider_info = provider.lock().await.get_provider_info().await.ok()?;
            let mut score = 0.0;
            // Calculate score based on provider graph
            if let Some(provider_scores) = provider_graph.get(provider_key) {
                for (capability, capability_score) in provider_scores {
                    if provider_info.capabilities.contains(capability) {
                        score += capability_score;
                    }
                }
            }
            // Calculate score based on user preference graph
            if let Some(user_preference_scores) = user_preference_graph.get(provider_key) {
                for (preference_key, preference_score) in user_preference_scores {
                    if provider_info.capabilities.contains(preference_key) {
                        score += preference_score;
                    }
                }
            }
            // Calculate score based on topic graph
            let topic_scores = topic_graph
                .iter()
                .filter(|(topic, _)| message.metadata.contains_key(topic))
                .flat_map(|(topic, scores)| scores.iter().map(move |(capability, score)| (topic, capability, score)))
                .filter(|(_, capability, _)| provider_info.capabilities.contains(capability))
                .map(|(_, _, score)| score)
                .sum::<f32>();
            score += topic_scores;
            scores.insert(provider_key.clone(), score);
        }
        let best_provider_key = scores
            .into_iter()
            .max_by(|(_, score1), (_, score2)| score1.partial_cmp(score2).unwrap())
            .map(|(key, _)| key);
        best_provider_key.and_then(|key| self.providers.get(&key).cloned())
    }
}

pub struct AIProviderManager {
    pub provider_selector: ProviderSelector,
    pub default_provider_key: String,
}

impl AIProviderManager {
    pub fn new(api_keys: HashMap<&str, (&str, &str)>, default_provider_key: &str) -> Self {
        let mut provider_selector = ProviderSelector::new();
        for (key, (api_key, base_url)) in api_keys {
            provider_selector.add_provider(
                key,
                AIProvider {
                    api_key: api_key.to_string(),
                    client: Client::new(),
                    base_url: base_url.to_string(),
                },
            );
        }
        Self {
            provider_selector,
            default_provider_key: default_provider_key.to_string(),
        }
    }

    async fn run_inference(&self, message: Message) -> Result<InferenceResponse, reqwest::Error> {
        let classification = classify_message(&message.metadata, &message.entity_graph);
        let criteria = HashMap::from([("capability".to_string(), classification)]);
        let provider = self.provider_selector.select_provider(&HashMap::new(), &HashMap::new(), &criteria, &message).await.unwrap_or_else(|| {
            self.provider_selector.providers.get(&self.default_provider_key).unwrap().clone()
        });
        let request = InferenceRequest {
            message,
            model: None,
            parameters: None,
        };
        provider.lock().await.run_inference(request).await
    }

    pub async fn run_generation(&self, message: Message) -> Result<GenerationResponse, reqwest::Error> {
        let classification = classify_message(&message.metadata, &message.entity_graph);
        let criteria = HashMap::from([("capability".to_string(), classification)]);
        let provider = self.provider_selector.select_provider(&HashMap::new(), &HashMap::new(), &criteria, &message).await.unwrap_or_else(|| {
            self.provider_selector.providers.get(&self.default_provider_key).unwrap().clone()
        });
        let request = GenerationRequest {
            message,
            max_length: None,
            temperature: None,
            n_best: None,
        };
        provider.lock().await.run_generation(request).await
    }

    pub fn add_provider(&mut self, name: &str, provider: AIProvider) {
    self.provider_selector.add_provider(name, provider);
}

}
