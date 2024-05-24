use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use actix_web::web::Data;
use async_graphql::connection::Edge;

use crate::messaging::message_metadata::MessageMetadata;
use crate::graphs::nl_to_graph::EntityGraphImpl;
use crate::messaging::decentralised_messaging::{Intent};
use crate::provider_types::payments::Payment;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub sender: String,
    pub recipient: String,
    pub content: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub edited_at: Option<chrono::DateTime<Utc>>,
    pub hash: String,
    pub metadata: MessageMetadata,
    pub feedback_weights: Vec<f32>,
    pub text: String,
    pub intent: Intent,
    pub payment: Option<Payment>,
    pub nonce: u64,
    pub name: String,
    pub data: Vec<Data<String>>,
    pub header: String,
    pub body: String,
    pub contexts: Vec<i32>,
    pub values: Vec<String>,
    pub entity_graph: EntityGraphImpl,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageBody {
    pub content: String,
    // Add other fields as needed
}

impl std::fmt::Display for MessageBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Implement the formatting logic for MessageBody
        // For example:
        write!(f, "MessageBody {{ /* fields */ }}")
    }
}