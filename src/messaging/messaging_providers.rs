use std::collections::HashMap;

use crate::messaging::message_metadata::MessageMetadata;
use crate::messaging::message::Message;
use crate::graphs::nl_to_graph::EntityGraph;


pub enum MessagingProviderType {
    Rest(Box<dyn MessagingProvider>),
    Grpc(Box<dyn MessagingProvider>),
    Webhooks(Box<dyn MessagingProvider>),
}

pub struct MessagingProviderFactory;

impl MessagingProviderFactory {
    pub fn create_messaging_provider(provider_type: &str, config: HashMap<String, String>) -> Result<MessagingProviderType, String> {
        match provider_type {
            "rest" => {
                let base_url = config.get("base_url")
                    .ok_or_else(|| "Missing base URL configuration for REST messaging provider".to_string())?
                    .to_string();
                Ok(MessagingProviderType::Rest(Box::new(RestMessagingProvider { base_url })))
            }
            "grpc" => {
                let endpoint = config.get("endpoint")
                    .ok_or_else(|| "Missing endpoint configuration for gRPC messaging provider".to_string())?
                    .to_string();
                Ok(MessagingProviderType::Grpc(Box::new(GrpcMessagingProvider { endpoint })))
            }
            "webhooks" => {
                let event_type = config.get("event_type")
                    .ok_or_else(|| "Missing event type configuration for Webhooks messaging provider".to_string())?
                    .to_string();
                Ok(MessagingProviderType::Webhooks(Box::new(WebhooksMessagingProvider { event_type })))
            }
            _ => Err("Unsupported messaging provider type".to_string()),
        }
    }
}

pub trait MessagingProvider {
    fn send_message(&self, recipient: &str, message: &str) -> Result<String, String>;
    fn subscribe_events(&self, callback_url: &str) -> Result<(), String>;
}

struct RestMessagingProvider {
    base_url: String,
}

impl MessagingProvider for RestMessagingProvider {
    fn send_message(&self, recipient: &str, message: &str) -> Result<String, String> {
        // Send a POST request to the messaging endpoint with the recipient and message
        // Return the message ID in the response body, or an error message if the request fails
        Ok("message_id".to_string())
    }

    fn subscribe_events(&self, callback_url: &str) -> Result<(), String> {
        // Send a POST request to the events endpoint with the callback URL
        // Return success or an error message if the request fails
        Ok(())
    }
}

struct GrpcMessagingProvider {
    endpoint: String,
}

impl MessagingProvider for GrpcMessagingProvider {
    fn send_message(&self, recipient: &str, message: &str) -> Result<String, String> {
        // Call the send_message method on the gRPC client with the recipient and message
        // Return the message ID in the response message, or an error message if the call fails
        Ok("message_id".to_string())
    }

    fn subscribe_events(&self, callback_url: &str) -> Result<(), String> {
        // Call the subscribe_events method on the gRPC client with the callback URL
        // Return success or an error message if the call fails
        Ok(())
    }
}

struct WebhooksMessagingProvider {
    event_type: String,
}

impl MessagingProvider for WebhooksMessagingProvider {
    fn send_message(&self, recipient: &str, message: &str) -> Result<String, String> {
        // Send a POST request to the webhook endpoint with the recipient, message, and event type
        // Return the message ID in the response body, or an error message if the request fails
        Ok("message_id".to_string())
    }

    fn subscribe_events(&self, callback_url: &str) -> Result<(), String> {
        // Send a POST request to the webhook endpoint with the callback URL and event type
        // Return success or an error message if the request fails
        Ok(())
    }
}
