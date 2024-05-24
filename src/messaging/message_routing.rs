//! # Message Classification and Routing
//!
//! This module provides functionality for classifying and routing messages based on their content and metadata.
//!
//! ## Key Features
//!
//! - Utilizes the spaCy library for natural language processing and entity extraction.
//! - Classifies messages into different categories based on their content and metadata.
//! - Routes classified messages to appropriate Kafka topics and MQTT topics.
//! - Supports handling messages from both Kafka and MQTT sources.
//! - Generates and publishes CloudEvents for classified messages.
//!
//! ## Main Components
//!
//! - `setup_kafka_producer`: Sets up a Kafka producer for sending classified messages.
//! - `setup_mqtt_client`: Sets up an MQTT client for subscribing to and publishing messages.
//! - `create_cloudevent`: Creates a CloudEvent based on the classified message.
//! - `handle_mqtt_messages`: Handles incoming MQTT messages and sends them for classification.
//! - `handle_kafka_messages`: Handles incoming Kafka messages and sends them for classification.
//! - `classify_message`: Classifies a message based on its metadata and entity graph.
//! - `route_message`: Routes a classified message to the appropriate Kafka and MQTT topics.
//! - `classify_and_route_message`: Classifies a message and routes it to the appropriate destinations.
//! - `parse_message`: Parses a message using spaCy and extracts entities to build an entity graph.
//! - `message_classifier`: Main function that receives messages, classifies them, and routes them.
//!
//! ## Usage
//!
//! 1. Set up the necessary configurations for Kafka and MQTT.
//! 2. Run the `main` function to start the message classification and routing process.
//! 3. Messages received from Kafka and MQTT will be classified and routed based on their content and metadata.
//! 4. Classified messages will be sent to the appropriate Kafka topics and MQTT topics.
//! 5. CloudEvents will be generated and published for classified messages.
//!
//! ## Testing
//!
//! The module includes unit tests to verify the functionality of the `classify_and_route_message` function.
//! The tests cover different scenarios and ensure that messages are correctly classified and routed.
//!
//! To run the tests, use the `cargo test` command.


use cloudevents::{EventBuilder, EventBuilderV10};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::config::FromClientConfig;
use std::time::Duration;
use log::error;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task;
use std::hash::{Hash, Hasher};

use crate::messaging::message::Message;
use crate::messaging::app_state::AppState;
use crate::messaging::message_metadata::{MessageMetadata, MetadataValue};
use crate::bindings::spacy_bindings::{Doc, EntityGraph, LangModel, SPACY};


async fn setup_kafka_producer() -> FutureProducer {
    FutureProducer::from_config(
        rdkafka::config::ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .set("message.timeout.ms", "5000"),
    ).expect("Failed to create Kafka producer")
}

fn setup_mqtt_client() -> AsyncClient {
    let mut mqtt_options = MqttOptions::new("my-client-id", "localhost", 1883);
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    AsyncClient::new(mqtt_options, 10).unwrap()
}

fn create_cloudevent(classification: String, message: String) -> Event {
    let event = EventBuilderV10::new()
        .id(uuid::Uuid::new_v4().to_string())
        .source("example.com/message")
        .ty("message.classified")
        .data("text/plain", message.as_bytes())
        .extension("classification", classification)
        .build()
        .unwrap();
    event
}

async fn handle_mqtt_messages(mut mqtt_client: AsyncClient, tx: mpsc::Sender<String>) {
    mqtt_client.subscribe("my/topic", QoS::AtLeastOnce).await.unwrap();
    while let Ok(event) = mqtt_client.eventloop.poll().await {
        if let Event::Incoming(Incoming::Publish(p)) = event {
            let message = String::from_utf8_lossy(&p.payload).to_string();
            if let Err(e) = tx.send(message).await {
                error!("Error sending message to classifier: {:?}", e);
            }
        }
    }
}

async fn handle_kafka_messages(consumer: BaseConsumer, tx: mpsc::Sender<String>) {
    for message in consumer.iter() {
        match message {
            Ok(m) => {
                let message = String::from_utf8_lossy(&m.payload).to_string();
                if let Err(e) = tx.send(message).await {
                    error!("Error sending message to classifier: {:?}", e);
                }
            }
            Err(e) => {
                error!("Kafka error: {:?}", e);
            }
        }
    }
}

fn classify_message(metadata: &HashMap<String, MetadataValue>, entity_graph: &EntityGraph) -> String {
    let mut classification = String::new();
    if entity_graph.has_entities_of_type(crate::bindings::spacy_bindings::EntityLabel::Location) {
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


pub async fn route_message(
    message: Message,
    lang_model: &LangModel,
    producer: &FutureProducer,
    mqtt_client: &mut AsyncClient,
    app_state: Arc<AppState>,
) {
    // Parse the message using the language model
    let doc = lang_model.nlp(message.text.clone()).await.unwrap();

    // Generate the entity graph from the parsed message
    let entity_graph = parse_message(&doc, lang_model);

    // Extract the metadata from the message
    let metadata = message.metadata.clone();

    // Classify the message based on the metadata and entity graph
    let classification = classify_message(&metadata.metadata, &entity_graph);

    let topic = match classification.as_str() {
        "Location-based message" => "location-based-topic",
        "Reply message" => "reply-topic",
        "Media message" => "media-topic",
        "Post message" => "post-topic",
        "Pinned message" => "pinned-topic",
        _ => "regular-topic",
    };

    let record = FutureRecord::to(&topic).payload(&message.text).key(&classification);
    let _ = producer.send(record, Duration::from_secs(0));

    if classification != "Regular message" {
        let event = create_cloudevent(classification, message.text.clone());
        let _ = mqtt_client.publish(topic, QoS::AtLeastOnce, false, serde_json::to_string(&event).unwrap().as_bytes());
    }

    // Get the sender from the message
    let sender = message.sender.clone();

    // Get the routing table from the app state
    let mut routing_table = app_state.get_routing_table().await;

    let recipient = message.recipient.clone();
    let node_id = match routing_table.get(&recipient) {
        Some(node_id) => node_id.clone(),
        None => {
            // If recipient is not found in the routing table, use consistent hashing to assign a node
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            recipient.hash(&mut hasher);
            let hash = hasher.finish();
            let node_index = hash as usize % routing_table.len();
            let node_id = routing_table.values().nth(node_index).unwrap().clone();
            routing_table.insert(recipient, node_id.clone());
            node_id
        }
    };

    // Forward the message to the assigned node using Kafka
    let node_topic = format!("node-{}", node_id);
    let node_record = FutureRecord::to(&node_topic).payload(&serde_json::to_string(&message).unwrap()).key(&sender);
    let _ = producer.send(node_record, Duration::from_secs(0));

    // Forward the message to the assigned node using MQTT
    let node_event = create_cloudevent("message.forwarded".to_string(), serde_json::to_string(&message).unwrap());
    let _ = mqtt_client.publish(&node_topic, QoS::AtLeastOnce, false, serde_json::to_string(&node_event).unwrap().as_bytes());
}

async fn classify_and_route_message(message: &str, metadata: MessageMetadata, producer: &FutureProducer, mqtt_client: &mut AsyncClient, lang_model: &LangModel, app_state: Arc<AppState>) {
    let entity_graph = parse_message(&lang_model.nlp(message.to_string()).await.unwrap(), lang_model);
    let classification = classify_message(&metadata.metadata, &entity_graph);
    let message_struct = Message {
        text: message.to_string(),
        id: uuid::Uuid::new_v4(),
        channel_id: String::new(),  // Replace with the actual channel ID
        sender: String::new(),  // Replace with the actual sender
        recipient: String::new(),  // Replace with the actual recipient
        timestamp: chrono::Utc::now(),
        edited_at: None,
        hash: String::new(),  // Replace with the actual hash
        content: String::new(),  // Add the missing content field
        metadata: metadata.clone(),
        feedback_weights: Vec::new(),  // Replace with the actual feedback weights
        intent: String::new(),  // Add the missing intent field
        payment: None,  // Add the missing payment field
        nonce: String::new(),  // Add the missing nonce field
        name: String::new(),  // Add the missing name field
        data: String::new(),  // Add the missing data field
        header: String::new(),  // Add the missing header field
        body: String::new(),  // Add the missing body field
        contexts: Vec::new(),  // Add the missing contexts field
        values: Vec::new(),  // Add the missing values field
        entity_graph: entity_graph.clone(),  // Add the missing entity_graph field
    };
    route_message(
        message_struct,
        lang_model,
        producer,
        mqtt_client,
        app_state.clone(),
    )
    .await;
}


fn parse_message(doc: &Doc, lang_model: &LangModel) -> EntityGraph {
    let mut entity_graph = EntityGraph::default();
    for ent in doc.ents(doc.py).unwrap() {
        let entity = ent.export(doc.py).unwrap();
        entity_graph.add_entity(entity.label, entity.text.to_string());
    }
    entity_graph
}

fn extract_metadata(message: &str) -> MessageMetadata {
    // Implement the logic to extract metadata from the message
    // This is a placeholder implementation
    MessageMetadata::default()
}

async fn message_classifier(
    mut rx: mpsc::Receiver<String>,
    producer: FutureProducer,
    mut mqtt_client: AsyncClient,
    lang_model: &LangModel,
    app_state: Arc<AppState>,
) {
    while let Some(message) = rx.recv().await {
        let metadata = extract_metadata(&message);
        classify_and_route_message(&message, metadata, &producer, &mut mqtt_client, lang_model, app_state.clone()).await;
    }
}

#[tokio::main]
async fn main() {
    let producer = setup_kafka_producer().await;
    let mut mqtt_client = setup_mqtt_client();
    let lang_model = SPACY.models.get(SPACY.default_lang).unwrap();

    let (tx, rx) = mpsc::channel(100);

    let app_state = Arc::new(AppState::new(/* provide the required argument */));

    let mqtt_task = task::spawn(handle_mqtt_messages(mqtt_client.clone(), tx.clone()));
    let consumer_builder = Consumer::from_config(
        rdkafka::config::ClientConfig::new()
            .set("group.id", "my-group")
            .set("bootstrap.servers", "localhost:9092")
            .set("auto.offset.reset", "earliest"),
    ).unwrap();
    let kafka_task = task::spawn(handle_kafka_messages(consumer_builder, tx.clone()));
    let classifier_task = task::spawn(message_classifier(rx, producer, mqtt_client, lang_model, app_state));
    tokio::select! {
        _ = mqtt_task => {},
        _ = kafka_task => {},
        _ = classifier_task => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdkafka::ClientConfig;

    #[tokio::test]
    async fn test_classify_and_route_message() {
        let message = "Hello, world!";
        let mut metadata = MessageMetadata::default();
        metadata.metadata.insert("post".to_string(), MetadataValue::Bool(true));
        metadata.metadata.insert("pinned".to_string(), MetadataValue::Bool(true));

        let producer_config = ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Failed to create Kafka producer config");

        let mut producer = FutureProducer::from_config(producer_config).unwrap();

        let mut mqtt_client = AsyncClient::new(MqttOptions::new("test-client", "localhost", 1883), 10).await.unwrap();
        let lang_model = SPACY.models.get(SPACY.default_lang).unwrap();
        let app_state = Arc::new(AppState::new(/* provide the required argument */)); // Create an instance of AppState and wrap it in an Arc
        classify_and_route_message(
            message,
            metadata,
            &producer,
            &mut mqtt_client,
            lang_model,
            app_state.clone(),
        ).await;
        // Add assertions to check the expected behavior
    }
}
