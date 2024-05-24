//! # DataBridging
//!
//! This module provides a flexible and extensible data bridging framework that allows for seamless integration
//! between different messaging systems, such as MQTT and Kafka. It defines a generic `DataBridge` trait that
//! can be implemented by various data bridging implementations to enable sending and receiving messages across
//! different systems.
//!
//! The module uses the CloudEvents specification for message formatting, ensuring compatibility and interoperability
//! with other systems that support CloudEvents.
//!
//! ## Functionality
//!
//! - `DataBridge` trait: Defines a generic interface for data bridging implementations. It includes the `send_message`
//! and `receive_message` methods, which are responsible for sending and receiving messages, respectively.
//!
//! - `MqttKafkaDataBridge`: An implementation of the `DataBridge` trait that combines MQTT and Kafka, allowing messages
//! to be sent via MQTT and received via Kafka. It takes a `BridgeConfig` struct as input, which contains the necessary
//! connection details for MQTT and Kafka.
//!
//! - `BridgeConfig`: A configuration struct that holds the connection details for the data bridging implementation,
//! such as the MQTT broker URL, MQTT topic, Kafka bootstrap servers, and Kafka topic.
//!
//! ## Benefits
//!
//! - Provides a flexible and extensible framework for data bridging, allowing for easy integration of different
//! messaging systems.
//!
//! - Defines a generic `DataBridge` trait that can be implemented by various data bridging implementations, enabling
//! a consistent interface for sending and receiving messages across different systems.
//!
//! - Uses the CloudEvents specification for message formatting, ensuring that the messages exchanged are well-structured
//! and can be easily understood by other systems that support CloudEvents.
//!
//! - Simplifies the process of working with multiple messaging systems by providing a unified interface to interact with
//! them while leveraging the benefits of the CloudEvents specification for message formatting and compatibility.

use cloudevents::Event;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use rumqttc::{Client, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Define the BridgeConfig struct
#[derive(Clone)]
pub struct BridgeConfig {
    pub mqtt_broker_url: String,
    pub mqtt_topic: String,
    pub kafka_bootstrap_servers: Vec<String>,
    pub kafka_topic: String,
}

// Define the DataBridge trait
pub trait DataBridge {
    fn send_message(&self, event: Event);
    fn receive_message(&self) -> Option<Event>;
}

pub struct MqttKafkaDataBridge {
    mqtt_client: Client,
    kafka_producer: FutureProducer,
    kafka_consumer: StreamConsumer,
    config: BridgeConfig,
}

impl MqttKafkaDataBridge {
    pub fn new(config: &BridgeConfig) -> Self {
        let mqtt_options = MqttOptions::new("mqtt-kafka-bridge", config.mqtt_broker_url.clone(), 1883)
            .set_keep_alive(Duration::from_secs(5));
        let mqtt_client = Client::new(mqtt_options, 10);

        let kafka_producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.kafka_bootstrap_servers.join(","))
            .create()
            .expect("Failed to create Kafka producer");

        let kafka_consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", "mqtt-kafka-bridge")
            .set("bootstrap.servers", &config.kafka_bootstrap_servers.join(","))
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .create()
            .expect("Failed to create Kafka consumer");

        kafka_consumer
            .subscribe(&[&config.kafka_topic])
            .expect("Failed to subscribe to Kafka topic");

        Self {
            mqtt_client,
            kafka_producer,
            kafka_consumer,
            config: config.clone(),
        }
    }
}

impl DataBridge for MqttKafkaDataBridge {
    fn send_message(&self, event: Event) {
        let message = serde_json::to_string(&event).expect("Failed to serialize CloudEvent");
        self.mqtt_client
            .publish(
                self.config.mqtt_topic.clone(),
                QoS::AtLeastOnce,
                false,
                message,
            )
            .unwrap();
    }

    fn receive_message(&self) -> Option<Event> {
        let result = self.kafka_consumer.poll(Duration::from_millis(100));
        match result {
            Some(Ok(message)) => {
                let payload = message
                    .payload()
                    .expect("Failed to get payload from Kafka message");
                let message_str =
                    std::str::from_utf8(payload).expect("Failed to convert payload to string");
                let event: Event =
                    serde_json::from_str(message_str).expect("Failed to deserialize CloudEvent");
                Some(event)
            }
            _ => None,
        }
    }
}