use cloudevents::{EventBuilder, EventBuilderV10};
use log::error;
use rdkafka::producer::FutureRecord;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use std::collections::HashMap;

use crate::messaging::message::Message;
use crate::messaging::message_routing::route_message;
use crate::data_streams::kafka::KafkaSink;
use crate::graphs::nl_to_graph::{EntityGraph, EntityType, EntityGraphImpl};
use crate::messaging::message_metadata::{MessageMetadata, MetadataValue};
use crate::utils::bigboterror::BigbotError;
use crate::messaging::app_state::AppState;


pub struct MessageRouter {
    kafka_sink: KafkaSink,
    mqtt_client: AsyncClient,
}

impl MessageRouter {
    pub async fn new(kafka_brokers: Vec<&str>, mqtt_broker: &str) -> Self {
        let producer = rdkafka::ClientConfig::new()
            .set("bootstrap.servers", kafka_brokers.join(","))
            .create()
            .expect("Failed to create Kafka producer");
        let kafka_sink = KafkaSink::new(producer, "your_topic".to_string());

        let mqtt_options = MqttOptions::new("my-client-id", mqtt_broker, 1883);
        let mqtt_client = AsyncClient::new(mqtt_options, 10).await.unwrap();

        Self {
            kafka_sink,
            mqtt_client,
        }
    }

    pub async fn route_message(&self, message: &Message, route: &str) -> Result<(), BigbotError> {
        // Implement the message routing logic here
        Ok(())
    }

    async fn start(&mut self) {
        self.mqtt_client.subscribe("my/topic", QoS::AtLeastOnce).await.unwrap();
        loop {
            match self.mqtt_client.try_next().await {
                Ok(event) => {
                    if let Incoming::Publish(p) = event {
                        let message = String::from_utf8(p.payload.to_vec()).unwrap();
                        let metadata = self.extract_metadata(&message);
                        self.classify_and_route_message(&message, &metadata).await;
                    }
                }
                Err(e) => {
                    error!("MQTT error: {:?}", e);
                }
            }
        }
    }

    fn extract_metadata(&self, message: &str) -> MessageMetadata {
        // Extract metadata from the message
        MessageMetadata::default()
    }

    async fn classify_and_route_message(&self, message: &str, metadata: &MessageMetadata) {
        let entity_graph = self.parse_message(message);
        let classification = self.classify_message(&metadata.metadata, &entity_graph);
        route_message(message.to_string(), classification, &self.kafka_sink, &mut self.mqtt_client, app_state).await;
    }

    fn parse_message(&self, message: &str) -> EntityGraphImpl {
        // Parse the message using spaCy and generate an entity graph
        EntityGraphImpl::new()
    }    

    fn classify_message(&self, metadata: &HashMap<String, MetadataValue>, entity_graph: &EntityGraphImpl) -> String {
        let mut classification = String::new();
        if entity_graph.has_entities_of_type(&EntityType::Location) {
            classification = "Location-based message".to_string();
        } else if let Some(MetadataValue::ReplyInfo(_)) = metadata.get("reply_to") {
            classification = "Reply message".to_string();
        } else if let Some(MetadataValue::MediaAttachment(_)) = metadata.get("media") {
            classification = "Media message".to_string();
        } else if let Some(MetadataValue::Bool(true)) = metadata.get("post") {
            classification = "Post message".to_string();
        } else if let Some(MetadataValue::Bool(true)) = metadata.get("pinned") {
            classification = "Pinned message".to_string();
        } else {
            classification = "Regular message".to_string();
        }
        classification
    }
}

#[tokio::main]
async fn main() {
    let kafka_brokers = vec!["localhost:9092"];
    let mqtt_broker = "localhost";
    let mut router = MessageRouter::new(kafka_brokers, mqtt_broker).await;
    router.start().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_classify_and_route_message() {
        let message = "Hello, world!";
        let mut metadata = MessageMetadata::default();
        metadata.metadata.insert("post".to_string(), MetadataValue::Bool(true));
        metadata.metadata.insert("pinned".to_string(), MetadataValue::Bool(true));
        let kafka_brokers = vec!["localhost:9092"];
        let mqtt_broker = "localhost";
        let router = MessageRouter::new(kafka_brokers, mqtt_broker).await;
        router.classify_and_route_message(message, &metadata).await;
        // Add assertions to check the behavior of the router
    }
}
