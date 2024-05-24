use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum DataItem {
    KeyValue { key: String, value: Value },
    Content { content: String, content_type: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageHeader {
    pub message_id: Uuid,
    pub mime_type: String,
    pub timestamp: String,
    pub source: Option<String>,
    pub destination: Option<String>,
    pub routing_key: Option<String>,
    pub correlation_id: Option<String>,
    pub reply_to: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    id: Uuid,
    name: String,
    data: Vec<DataItem>,
    header: MessageHeader,
}

#[derive(Debug)]
enum DecodeError {
    MissingField(String),
    InvalidField(String),
}

impl std::fmt::Display for MessageHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Implement the formatting logic for MessageHeader
        // For example:
        write!(f, "MessageHeader {{ /* fields */ }}")
    }
}

impl TryFrom<Message> for HashMap<String, Value> {
    type Error = DecodeError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let mut result = HashMap::new();

        for item in value.data {
            match item {
                DataItem::KeyValue { key, value } => {
                    result.insert(key, value);
                }
                DataItem::Content { content, content_type } => {
                    result.insert("content".to_string(), Value::String(content));
                    result.insert("content_type".to_string(), Value::String(content_type));
                }
            }
        }

        Ok(result)
    }
}

#[test]
fn run_request_adapter_example() {
    // Deserialize JSON data into a Message instance
    let json_str = r#"
    {
        "id": "a8098c1a-f86e-11da-bd1a-00112444be1e",
        "name": "ExampleMessage",
        "data": [
            {"key": "amount", "value": 50},
            {"key": "recipient", "value": "Sam Tay"},
            {"content": "This is the content", "content_type": "text/plain"}
        ],
        "header": {
            "message_id": "a8098c1a-f86e-11da-bd1a-00112444be1e",
            "mime_type": "application/json",
            "timestamp": "2023-05-24T10:30:00Z"
        }
    }
    "#;
    let message: Message = serde_json::from_str(json_str).unwrap();
    println!("{:?}", message);

    // Transform the extracted data into a HashMap
    let data: HashMap<String, Value> = message.try_into().unwrap();
    println!("{:?}", data);
}