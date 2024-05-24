//! # Topics Module
//!
//! This module defines constants and a function for generating Kafka topic names used in the data exchange pipeline.
//!
//! ## Topic Constants
//!
//! The module provides a set of constants representing various Kafka topics used in the system, including:
//!
//! - `INPUT_TOPIC`: The topic for incoming data.
//! - `PRE_PROCESSING_TOPIC`: The topic for data that needs pre-processing.
//! - `INFERENCE_TOPIC_PREFIX`: The prefix for topics related to inference tasks.
//! - `DELEGATE_TOPIC_PREFIX`: The prefix for topics related to delegation tasks.
//! - `AGENT_TOPIC`: The topic for agent-related messages.
//! - `GRAPH_TOPIC_PREFIX`: The prefix for topics related to graph processing.
//! - `SESSION_TOPIC_PREFIX`: The prefix for topics related to session management.
//! - `POST_PROCESSING_TOPIC`: The topic for data that needs post-processing.
//! - `RESULT_TOPIC`: The topic for publishing final results.
//! - `MONITORING_TOPIC`: The topic for monitoring and health check messages.
//! - `LOGGING_TOPIC`: The topic for logging and auditing purposes.
//! - `CONFIGURATION_TOPIC`: The topic for distributing configuration updates.
//!
//! ## Dynamic Topic Generation
//!
//! The module also provides a `dynamic_topic` function that generates a dynamic Kafka topic name
//! by concatenating a topic prefix and a topic suffix. This is useful for creating topic names
//! dynamically based on runtime data, such as session IDs or user IDs.
//!
//! The `dynamic_topic` function takes two parameters:
//!
//! - `topic_prefix`: The prefix of the topic name.
//! - `topic_suffix`: The suffix of the topic name.
//!
//! The function returns a new `String` representing the dynamically generated topic name.
//!
//! ## Usage
//!
//! To use the topics defined in this module, simply import the desired constants or the `dynamic_topic` function
//! into your Rust code and refer to them as needed.
//!
//! For example:
//!
//! ```rust
//! use crate::data_exchange::topics::{INPUT_TOPIC, RESULT_TOPIC, dynamic_topic};
//!
//! println!("Input Topic: {}", INPUT_TOPIC);
//! println!("Result Topic: {}", RESULT_TOPIC);
//!
//! let session_id = "123";
//! let session_topic = dynamic_topic(SESSION_TOPIC_PREFIX, session_id);
//! println!("Session Topic: {}", session_topic);
//! ```

pub const INPUT_TOPIC: &str = "input-topic";
pub const PRE_PROCESSING_TOPIC: &str = "pre-processing-topic";
pub const INFERENCE_TOPIC_PREFIX: &str = "inference-topic";
pub const DELEGATE_TOPIC_PREFIX: &str = "delegate-topic";
pub const AGENT_TOPIC: &str = "agent-topic";
pub const GRAPH_TOPIC_PREFIX: &str = "graph-topic";
pub const SESSION_TOPIC_PREFIX: &str = "session-topic";
pub const POST_PROCESSING_TOPIC: &str = "post-processing-topic";
pub const RESULT_TOPIC: &str = "result-topic";
pub const MONITORING_TOPIC: &str = "monitoring-topic";
pub const LOGGING_TOPIC: &str = "logging-topic";
pub const CONFIGURATION_TOPIC: &str = "configuration-topic";

pub fn dynamic_topic(topic_prefix: &str, topic_suffix: &str) -> String {
    format!("{}-{}", topic_prefix, topic_suffix)
}
