//! # Messaging Module
//!
//! This module provides the core messaging functionality and data structures for the application.
//!
//! ## Message
//!
//! The `Message` struct represents a message with various fields like `id`, `channel_id`, `sender`, `recipient`, `content`, `timestamp`, etc.
//!
//! ## Channel
//!
//! The `Channel` struct represents a communication channel with an `id`, `name`, and a list of `messages`.
//!
//! ## ChannelStore
//!
//! The `ChannelStore` struct provides methods for interacting with the database to perform various operations related to channels and messages.
//!
//! ### Methods
//!
//! - `new`: Creates a new instance of `ChannelStore` with a database connection.
//! - `create_channel`: Creates a new channel with the given name and message hash batch size.
//! - `send_message`: Sends a message to a specific channel with the provided details.
//! - `edit_message`: Edits the content of a message identified by its ID.
//! - `get_messages`: Retrieves messages for a specific channel and recipient.
//! - `validate_message`: Validates the integrity of a message by comparing its stored hash with the computed hash.
//!
//! ## Messaging Handler
//!
//! The `messaging_handler` module defines the `MessagingHandler` struct, which handles the sending of messages through different messaging protocols based on the channel state.
//!
//! ### Methods
//!
//! - `new`: Creates a new instance of `MessagingHandler` with the provided messaging protocol instances.
//! - `send`: Sends a message using the appropriate messaging protocol based on the channel state.
//!
//! ### Channel State
//!
//! The `ChannelState` enum represents the different states of a channel, which determines the messaging protocol to be used for sending messages.
//!
//! - `Active`: Indicates that the channel is active, and messages should be sent using WebSocket.
//! - `Inactive`: Indicates that the channel is inactive, and messages should be sent using Kafka.
//! - `LowBandwidth`: Indicates that the channel has low bandwidth, and messages should be sent using MQTT.
//! - `Nats`: Indicates that messages should be sent using NATS.
//!
//! ## Run Function
//!
//! The `run` function sets up the Ockam context, initializes the necessary components, and demonstrates the usage of the messaging functionality.
//!
//! It creates instances of the messaging protocols (WebSocket, Kafka, MQTT, NATS) and the `MessagingHandler`. It then sends messages using the `MessagingHandler` with different channel states.

use crate::encryption::encryption::{encrypt_message, decrypt_message, hash_message, EncryptHandler};
use crate::messaging::decentralised_messaging::Intent;
use crate::messaging::message::{Message, MessageBody};
use crate::messaging::message_metadata::MessageMetadata;
use crate::messaging::pii_handler::PIIHandler;
use crate::messaging::consensus::ConsensusLayer;
use crate::messaging::route_classifier::MessageRouter;
use crate::clients::kv::{MemoryKVStore, PrefixedKVStore, KVStore};
use crate::messaging::app_state::AppState;
use crate::utils::bigboterror::BigbotError;
use crate::provider_types::payments::Payment;
use crate::data_exchange::exchange_adapters::MessageHeader;
use ockam::Context;
use crate::graphs::nl_to_graph::EntityGraph;

use chrono::Utc;
use tikv_client::{RawClient, TransactionClient, BoundRange};
use serde::Serialize;
use uuid::Uuid;
use std::time::Duration;
use std::sync::Arc;
use nats::Connection;
use kafka::producer::{Producer, RequiredAcks};
use kafka::client::Compression;
use kafka::consumer::{Consumer, FetchOffset};
use rdkafka::producer::FutureRecord;
use serde_json::Value;

#[derive(Serialize)]
struct Channel {
    id: Uuid,
    name: String,
    pub messages: Vec<Message>,
    message_hash_batch_size: usize,
}

#[derive(Clone)]
struct ChannelStore {
    raw_client: RawClient,
    txn_client: TransactionClient,
}

pub struct RouteClassifier;

impl RouteClassifier {
    pub fn new() -> Self {
        Self
    }

    pub fn classify(&self, message: &Message) -> Result<String, BigbotError> {
        // Implement route classification logic here
        Ok("route".to_string())
    }
}

impl ChannelStore {
    async fn new(pd_endpoints: &[String]) -> Result<Self, BigbotError> {
        let raw_client = RawClient::new(pd_endpoints.to_vec()).await.map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
        let txn_client = TransactionClient::new(pd_endpoints.to_vec()).await.map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
        Ok(Self { raw_client, txn_client })
    }

    async fn get_message(&self, message_id: Uuid) -> Result<Message, BigbotError> {
        let key = format!("/messages/{}", message_id);
        let value = self.raw_client.get(key).await.map_err(|e| BigbotError::DatabaseError(e.to_string()))?.ok_or(BigbotError::InvalidInput("Message not found".to_string()))?;
        let message: Message = serde_json::from_slice(&value[..]).map_err(|e| BigbotError::InvalidInput(e.to_string()))?;
        Ok(message)
    }

    async fn create_channel(&self, name: &str, message_hash_batch_size: usize) -> Result<Channel, BigbotError> {
        let id = Uuid::new_v4();
        let channel = Channel {
            id,
            name: name.to_string(),
            messages: Vec::new(),
            message_hash_batch_size,
        };
        let key = format!("/channels/{}", id);
        let value = serde_json::to_string(&channel).map_err(|e| BigbotError::InvalidInput(e.to_string()))?;
        self.raw_client.put(key, value).await.map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
        Ok(channel)
    }

    pub async fn send_message(
        &self,
        channel_id: Uuid,
        sender: &str,
        recipient: &str,
        content: &str,
        metadata: MessageMetadata,
        feedback_weights: Vec<f32>,
        text: String,
        intent: String,
        payment: Option<Payment>,
        nonce: String,
        name: String,
        data: Vec<u8>,
        header: MessageHeader,
        body: MessageBody,
        contexts: Vec<Context>,
        values: Vec<Value>,
        entity_graph: &impl EntityGraph,
    ) -> Result<Message, BigbotError> {
        let message = Message {
            id: Uuid::new_v4(),
            channel_id,
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            edited_at: None,
            metadata,
            feedback_weights,
            text,
            intent: Intent::from(intent),
            payment,
            nonce: nonce.parse().unwrap(),
            name,
            data: data.into_iter().map(|d| actix_web::web::Data::new(d.to_string())).collect(),
            header: header.to_string(),
            body: body.to_string(),
            contexts: contexts.into_iter().map(|_| 0).collect(),
            values: values.into_iter().map(|v| v.to_string()).collect(),
            entity_graph: entity_graph.clone(),
            hash: String::new(),
        };
        let encrypted_content = encrypt_message(&message.content, &message.recipient).map_err(|e| BigbotError::NlpError(e.to_string()))?;
        let hash = hash_message(&encrypted_content).map_err(|e| BigbotError::NlpError(e.to_string()))?;
        let mut message_with_hash = message.clone();
        message_with_hash.content = encrypted_content;
        message_with_hash.hash = hash;
        let key = format!("/messages/{}/{}", channel_id, message_with_hash.id);
        let value = serde_json::to_string(&message_with_hash).map_err(|e| BigbotError::InvalidInput(e.to_string()))?;
        self.raw_client.put(key, value).await.map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
        Ok(message)
    }

    async fn edit_message(
        &self,
        message_id: Uuid,
        content: &str,
        use_pessimistic_txn: bool,
    ) -> Result<Message, BigbotError> {
        let key = format!("/messages/{}", message_id);
        let txn_result = if use_pessimistic_txn {
            self.txn_client.begin_pessimistic().await
        } else {
            self.txn_client.begin_optimistic().await
        };
    
        let mut txn = txn_result.map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
    
        let value = txn.get(key.clone()).await.map_err(|e| BigbotError::DatabaseError(e.to_string()))?.ok_or(BigbotError::InvalidInput("Message not found".to_string()))?;
        let mut message: Message = serde_json::from_str(&String::from_utf8_lossy(&value)).map_err(|e| BigbotError::InvalidInput(e.to_string()))?;
        let encrypted_content = encrypt_message(content, &message.recipient).map_err(|e| BigbotError::NlpError(e.to_string()))?;
        let hash = hash_message(&encrypted_content).map_err(|e| BigbotError::NlpError(e.to_string()))?;
        message.content = encrypted_content;
        message.edited_at = Some(Utc::now());
        message.hash = hash;
        let value = serde_json::to_string(&message).map_err(|e| BigbotError::InvalidInput(e.to_string()))?;
        txn.put(key, value).await.map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
    
        match txn.commit().await {
            Ok(_) => Ok(message),
            Err(e) if !use_pessimistic_txn && e.to_string().contains("TxnAbortedError") => {
                // Retry the transaction using Box::pin
                Box::pin(self.edit_message(message_id, content, use_pessimistic_txn)).await
            }
            Err(e) => Err(BigbotError::DatabaseError(e.to_string())),
        }
    }    

    async fn get_messages(
        &self,
        channel_id: Uuid,
        recipient: &str,
    ) -> Result<Vec<Message>, BigbotError> {
        let prefix = format!("/messages/{}/", channel_id);
        let mut messages = Vec::new();
        let bound_range = BoundRange::from(prefix.clone()..);
        let kv_pairs = self.raw_client.scan(bound_range, u32::MAX).await.map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
        for kv_pair in kv_pairs {
            let _key = kv_pair.key();
            let value = kv_pair.value();
            let message: Message = serde_json::from_str(&String::from_utf8_lossy(&value)).map_err(|e| BigbotError::InvalidInput(e.to_string()))?;
            if message.recipient == recipient {
                let decrypted_content = decrypt_message(&message.content, recipient).map_err(|e| BigbotError::NlpError(e.to_string()))?;
                let mut decrypted_message = message.clone();
                decrypted_message.content = decrypted_content;
                messages.push(decrypted_message);
            }
        }
        Ok(messages)
    }        

    async fn validate_message(&self, message_id: Uuid) -> Result<bool, BigbotError> {
        let key = format!("/messages/{}", message_id);
        let value = self.raw_client.get(key).await.map_err(|e| BigbotError::DatabaseError(e.to_string()))?.ok_or(BigbotError::InvalidInput("Message not found".to_string()))?;
        let message: Message = serde_json::from_str(&String::from_utf8_lossy(&value)).map_err(|e| BigbotError::InvalidInput(e.to_string()))?;
        let computed_hash = hash_message(&message.content).map_err(|e| BigbotError::NlpError(e.to_string()))?;
        Ok(message.hash == computed_hash)
    }
}

pub mod messaging_handler {
    use super::*;

    pub struct MessagingHandler {
        kafka_producer: Producer,
        nats: Arc<Connection>,
    }

    impl MessagingHandler {
        pub fn new(
            kafka_producer: Producer,
            nats: Arc<Connection>,
        ) -> Self {
            MessagingHandler {
                kafka_producer,
                nats,
            }
        }

        pub fn send(&self, message: &Message, channel_state: ChannelState) -> Result<(), BigbotError> {
            match channel_state {
                ChannelState::Active => {
                    let future_record = FutureRecord::to(&message.channel_id.to_string())
                        .payload(&serde_json::to_string(&message).map_err(|e| BigbotError::InvalidInput(e.to_string()))?)
                        .key(&message.id.to_string());
                    self.kafka_producer.send(future_record).map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
                }
                ChannelState::Inactive => {
                    // Handle inactive channel state, e.g., log a warning or error
                    eprintln!("Warning: Attempting to send a message to an inactive channel");
                }
                ChannelState::LowBandwidth => {
                    // Handle low bandwidth channel state, e.g., use MQTT or a different protocol
                    // TODO: Implement low bandwidth message sending logic
                    unimplemented!("Low bandwidth message sending is not implemented yet");
                }
                ChannelState::Nats => self.nats.publish(&message.channel_id.to_string(), message.content.as_bytes()).map_err(|e| BigbotError::DatabaseError(e.to_string()))?,
            }
            Ok(())
        }
    }

    pub enum ChannelState {
        Active,
        Inactive,
        LowBandwidth,
        Nats,
    }
}

pub struct MessagingApp {
    channel_store: ChannelStore,
    messaging_handler: messaging_handler::MessagingHandler,
    message_router: MessageRouter,
    pii_handler: PIIHandler,
    route_classifier: RouteClassifier,
    consensus_layer: Option<ConsensusLayer>,
    use_pessimistic_txn: bool,
}

impl MessagingApp {
    pub async fn new(
        tikv_endpoints: &[String],
        local_storage_path: &str,
        distributed_hash_endpoints: &[String],
        enable_consensus: bool,
        encrypt_handler: Arc<EncryptHandler>,
        use_pessimistic_txn: bool,
        kafka_brokers: Vec<&str>,
        mqtt_broker: &str,
    ) -> Result<Self, BigbotError> {
        let channel_store = ChannelStore::new(tikv_endpoints).await?;
        let kafka_producer = Producer::from_hosts(vec!["localhost:9092".to_owned()])
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
            .with_compression(Compression::NONE)
            .create()
            .map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
        let nats = Arc::new(nats::connect("nats://localhost:4222").map_err(|e| BigbotError::DatabaseError(e.to_string()))?);
        let messaging_handler = messaging_handler::MessagingHandler::new(kafka_producer, nats);
        let app_state = Arc::new(AppState::new());
        let consensus_layer = if enable_consensus {
            Some(ConsensusLayer::new(tikv_endpoints, local_storage_path, distributed_hash_endpoints, app_state).await?)
        } else {
            None
        };
        let message_router = MessageRouter::new(kafka_brokers, mqtt_broker).await;
        let pii_handler = PIIHandler::new(encrypt_handler);
        let route_classifier = RouteClassifier::new();
        Ok(Self {
            channel_store,
            messaging_handler,
            message_router,
            pii_handler,
            route_classifier,
            consensus_layer,
            use_pessimistic_txn,
        })
    }

    pub async fn create_channel(&self, name: &str, message_hash_batch_size: usize) -> Result<Channel, BigbotError> {
        self.channel_store.create_channel(name, message_hash_batch_size).await
    }

    pub async fn send_message(
        &self,
        channel_id: Uuid,
        sender: &str,
        recipient: &str,
        content: &str,
        metadata: MessageMetadata,
        feedback_weights: Vec<f32>,
        text: String,
        intent: String,
        payment: Option<Payment>,
        nonce: String,
        name: String,
        data: Vec<u8>,
        header: MessageHeader,
        body: MessageBody,
        contexts: Vec<Context>,
        values: Vec<Value>,
        entity_graph: &impl EntityGraph,
    ) -> Result<Message, BigbotError> {
        let message = self.channel_store.send_message(
            channel_id,
            sender,
            recipient,
            content,
            metadata,
            feedback_weights,
            text,
            intent,
            payment,
            nonce,
            name,
            data,
            header,
            body,
            contexts,
            values,
            entity_graph,
        ).await?;
        if let Some(consensus_layer) = &self.consensus_layer {
            if !consensus_layer.validate_message(&message).await? {
                return Err(BigbotError::InvalidInput("Message validation failed".into()));
            }
            consensus_layer.replicate_message(&message).await?;
        }
        self.messaging_handler.send(&message, messaging_handler::ChannelState::Active)?;
        Ok(message)
    }

    pub async fn edit_message(
        &self,
        message_id: Uuid,
        content: &str,
    ) -> Result<Message, BigbotError> {
        self.channel_store.edit_message(message_id, content, self.use_pessimistic_txn).await
    }

    pub async fn get_message(&self, message_id: Uuid) -> Result<Message, BigbotError> {
        self.channel_store.get_message(message_id).await
    }

    pub async fn validate_message(&self, message_id: Uuid) -> Result<bool, BigbotError> {
        self.channel_store.validate_message(message_id).await
    }

    pub async fn sync_messages(&self) -> Result<(), BigbotError> {
        if let Some(consensus_layer) = &self.consensus_layer {
            consensus_layer.sync_messages().await?;
        }
        Ok(())
    }

    pub async fn process_message(
        &self,
        message: &Message,
    ) -> Result<(), BigbotError> {
        // Apply PII handling
        let sanitized_message = self.pii_handler.sanitize(message)?;
        // Classify the message route
        let route = self.route_classifier.classify(&sanitized_message)?;
        // Route the message
        self.message_router.route_message(&sanitized_message, &route).await?;
        Ok(())
    }
}

pub async fn run(
    tikv_endpoints: &[String],
    local_storage_path: &str,
    distributed_hash_endpoints: &[String],
    enable_consensus: bool,
    use_pessimistic_txn: bool,
    kafka_brokers: Vec<&str>,
    mqtt_broker: &str,
) -> Result<(), BigbotError> {

    // Initialize the encryption handler
    let store = Arc::new(MemoryKVStore::default());
    let keyid_store = Arc::new(PrefixedKVStore::new(store.clone() as Arc<dyn KVStore>, "OCKAM_KEYID:".into()));
    let encrypt_handler = Arc::new(EncryptHandler::new(keyid_store));
    
    // Create an instance of MessagingApp
    let messaging_app = match MessagingApp::new(
        tikv_endpoints,
        local_storage_path,
        distributed_hash_endpoints,
        enable_consensus,
        encrypt_handler,
        use_pessimistic_txn,
        kafka_brokers,
        mqtt_broker,
    ).await {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Failed to create MessagingApp: {:?}", e);
            return Err(e);
        }
    };

    // Create a new Kafka consumer
    let mut kafka_consumer = Consumer::from_hosts(vec!["localhost:9092".to_owned()])
        .with_topic("ockam_key_exchange".to_owned())
        .with_group("my-group".to_owned())
        .with_fallback_offset(FetchOffset::Earliest)
        .create()
        .map_err(|e| BigbotError::DatabaseError(e.to_string()))?;

    // Receive messages from Kafka
    loop {
        let mut messages = kafka_consumer.poll().map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
        for m in messages.iter_mut() {
            if let Some(value) = m.messages_mut() {
                let message: Message = serde_json::from_slice(value).map_err(|e| BigbotError::InvalidInput(e.to_string()))?;
                messaging_app.process_message(&message).await?;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), BigbotError> {
    let tikv_endpoints = vec!["127.0.0.1:2379".to_string()];
    let local_storage_path = "./local_storage";
    let distributed_hash_endpoints = vec!["127.0.0.1:6000".to_string()];
    let enable_consensus = true;
    let use_pessimistic_txn = false; // Set to true for pessimistic transactions
    let kafka_brokers = vec!["localhost:9092"]; // Replace with your Kafka broker addresses
    let mqtt_broker = "localhost"; // Replace with your MQTT broker address

    run(
        &tikv_endpoints,
        local_storage_path,
        &distributed_hash_endpoints,
        enable_consensus,
        use_pessimistic_txn,
        kafka_brokers,
        mqtt_broker,
    ).await?;
    
    Ok(())
}
