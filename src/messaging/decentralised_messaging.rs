use iroh::client::{Iroh};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use etcd_client::{Client, KvClient, WatchClient, WatchResponse};
use tokio::sync::mpsc::{Sender, Receiver};
use uuid::Uuid;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use bytes::Bytes;

use crate::messaging::message::Message;

// Define your custom client backend
struct CustomClient {
    // Implement the necessary fields and methods
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Intent {
    TextMessage,
    Payment,
    GroupInvitation,
    // Add more intents as needed
}

struct AppState {
    etcd_client: Arc<RwLock<KvClient>>,
    watch_client: Arc<RwLock<WatchClient>>,
    node_id: String,
    staked_tokens: Arc<RwLock<HashMap<String, u64>>>,
    nonce_counter: Arc<RwLock<u64>>,
    iroh_client: Arc<RwLock<Iroh<CustomClient>>>,
    routing_table: Arc<RwLock<HashMap<String, String>>>,
}

async fn send_message(state: Arc<AppState>, message: Message) -> Result<(), String> {
    let mut iroh_client = state.iroh_client.write().await;
    let message_json = serde_json::to_string(&message).map_err(|e| e.to_string())?;
    let cid = iroh_client.put(Bytes::from(message_json)).await.map_err(|e| e.to_string())?;

    let mut etcd_client = state.etcd_client.write().await;
    etcd_client.put(cid.clone(), message.id.clone(), None).await.map_err(|e| e.to_string())?;

    Ok(())
}

async fn get_message(state: Arc<AppState>, message_id: &str) -> Result<Option<Message>, String> {
    let etcd_client = state.etcd_client.read().await;
    let response = etcd_client.get(message_id, None).await.map_err(|e| e.to_string())?;
    if let Some(kv) = response.kvs().first() {
        let cid = kv.value_str().to_string();
        let iroh_client = state.iroh_client.read().await;
        let message_json = iroh_client.get(&cid).await.map_err(|e| e.to_string())?;
        let message: Message = serde_json::from_slice(&message_json).map_err(|e| e.to_string())?;
        Ok(Some(message))
    } else {
        Ok(None)
    }
}

async fn watch_messages(state: Arc<AppState>, mut tx: Sender<Message>) -> Result<(), String> {
    let mut watch_client = state.watch_client.write().await;
    let (watcher, mut stream) = watch_client.watch("", None).await.map_err(|e| format!("{}", e))?;

    tokio::spawn(async move {
        while let Some(resp) = stream.message().await.map_err(|e| format!("{}", e)).unwrap() {
            if resp.canceled() {
                break;
            }
            for event in resp.events() {
                if let Some(kv) = event.kv() {
                    let message_id = kv.key_str().to_string();
                    if let Ok(Some(message)) = get_message(state.clone(), &message_id).await {
                        tx.send(message).await.unwrap();
                    }
                }
            }
        }
    });

    Ok(())
}

async fn handle_message(state: Arc<AppState>, message: Message) {
    let _node_id = state.node_id.clone();
    let mut staked_tokens = state.staked_tokens.write().await;
    let mut nonce_counter = state.nonce_counter.write().await;

    // Check if the sender has sufficient staked tokens
    let sender_tokens = staked_tokens.get(&message.sender).cloned().unwrap_or(0);
    let required_tokens = match &message.payment {
        Some(payment) => payment.amount.parse::<u64>().unwrap_or(0),
        None => 0,
    };

    if sender_tokens >= required_tokens {
        // Check if the message nonce is valid
        if message.nonce == *nonce_counter + 1 {
            // Process the message based on its intent
            match message.intent {
                Intent::TextMessage => {
                    println!("Node {}: Received text message: {:?}", _node_id, message);
                    // Handle text message
                }
                Intent::Payment => {
                    println!("Node {}: Received payment message: {:?}", _node_id, message);
                    // Handle payment message
                }
                Intent::GroupInvitation => {
                    println!("Node {}: Received group invitation message: {:?}", _node_id, message);
                    // Handle group invitation message
                }
            }

            // Update the nonce counter
            *nonce_counter += 1;
        } else {
            println!("Node {}: Invalid nonce. Expected: {}, Received: {}", _node_id, *nonce_counter + 1, message.nonce);
        }
    } else {
        println!("Node {}: Insufficient staked tokens. Required: {}, Available: {}", _node_id, required_tokens, sender_tokens);
    }
}

async fn stake_tokens(state: Arc<AppState>, sender: String, amount: u64) {
    let mut staked_tokens = state.staked_tokens.write().await;
    let current_stake = staked_tokens.entry(sender).or_insert(0);
    *current_stake += amount;
    println!("Staked {} tokens for sender: {}", amount, sender);
}

async fn route_message(state: Arc<AppState>, message: Message) -> Result<(), String> {
    let mut routing_table = state.routing_table.write().await;
    let recipient = message.recipient.clone();
    let _node_id = match routing_table.get(&recipient) {
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

    // Forward the message to the assigned node
    // Implement the logic to send the message to the assigned node

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let etcd_endpoints = vec!["http://localhost:2379".to_string()];
    let etcd_client = Client::connect(etcd_endpoints.clone(), None).await?;
    let kv_client = etcd_client.kv_client();
    let watch_client = etcd_client.watch_client();

    let node_id = Uuid::new_v4().to_string();
    let staked_tokens = Arc::new(RwLock::new(HashMap::new()));
    let nonce_counter = Arc::new(RwLock::new(0));

    let custom_client = CustomClient {};
    let iroh_client = Arc::new(RwLock::new(Iroh::new(custom_client)));

    let routing_table = Arc::new(RwLock::new(HashMap::new()));

    let app_state = Arc::new(AppState {
        etcd_client: Arc::new(RwLock::new(kv_client)),
        watch_client: Arc::new(RwLock::new(watch_client)),
        node_id,
        staked_tokens,
        nonce_counter,
        iroh_client,
        routing_table,
    });

    let (tx, mut rx): (Sender<Message>, Receiver<Message>) = tokio::sync::mpsc::channel(32);

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Err(e) = route_message(app_state.clone(), message.clone()).await {
                println!("Error routing message: {}", e);
            }
            handle_message(app_state.clone(), message).await;
        }
    });

    watch_messages(app_state.clone(), tx).await?;

    // Example usage
    let sender = "Alice".to_string();
    let recipient = "Bob".to_string();

    // Stake tokens for the sender
    stake_tokens(app_state.clone(), sender.clone(), 100).await;

    let message = Message {
        id: Uuid::new_v4().to_string(),
        sender: sender.clone(),
        recipient,
        timestamp: chrono::Utc::now().to_rfc3339(),
        content: "Hello, Bob!".to_string(),
        intent: Intent::TextMessage,
        payment: None,
        nonce: *app_state.nonce_counter.read().await + 1,
        channel_id: None,
        edited_at: None,
        hash: None,
        metadata: None,
        feedback_weights: None,
        text: None,
        name: None,
        data: None,
        header: None,
        body: None,
        contexts: None,
        values: None,
        entity_graph: None,
    };

    send_message(app_state.clone(), message).await?;

    tokio::signal::ctrl_c().await?;
    println!("Shutting down gracefully...");

    Ok(())
}
