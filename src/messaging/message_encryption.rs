use ockam::{Context, Result, Worker};
use ockam_core::AsyncTryClone;
use ockam_transport_websocket::WebSocketTransport;
use serde::{Deserialize, Serialize};

use crate::messaging::message::Message;

// Define a WebSocketMessage struct
#[derive(Serialize, Deserialize)]
struct WebSocketMessage {
    payload: Vec<u8>,
    addr: String,
}

impl ockam::Message for WebSocketMessage {}

impl WebSocketMessage {
    fn new(payload: Vec<u8>, addr: String) -> Self {
        Self { payload, addr }
    }
}

// Define a Vault struct for encryption and decryption
struct Vault;

impl Vault {
    fn create(_ctx: &Context) -> Result<Self> {
        Ok(Self)
    }

    fn decrypt(&self, _data: &[u8]) -> Result<Vec<u8>> {
        // Implement decryption logic here
        Ok(vec![])
    }

    fn encrypt(&self, _recipient: &str, _data: &[u8]) -> Result<Vec<u8>> {
        // Implement encryption logic here
        Ok(vec![])
    }

    fn secret_generate(&self) -> Result<Vec<u8>> {
        // Implement secret generation logic here
        Ok(vec![])
    }

    fn ec_diffie_hellman(&self, _key_pair: &[u8], _remote_public_key: &[u8]) -> Result<Vec<u8>> {
        // Implement EC Diffie-Hellman key exchange logic here
        Ok(vec![])
    }

    async fn async_try_clone(&self) -> Result<Self> {
        Ok(Self)
    }
}

// Define a worker for handling encrypted messages
struct EncryptedMessageHandler {
    vault: Vault,
}

impl Worker for EncryptedMessageHandler {
    type Message = WebSocketMessage;
    type Context = Context;

    async fn handle_message(&mut self, ctx: &mut Self::Context, msg: Self::Message) -> ockam::Result<()> {
        // Decrypt the incoming message body using Vault
        let decrypted = self.vault.decrypt(msg.payload.as_slice())?;

        // Deserialize the decrypted message into a Message struct
        let message: Message = serde_json::from_slice(&decrypted).map_err(|e| ockam::Error::new(ockam::errcode::Origin::Application, ockam::errcode::Kind::Invalid, e))?;

        // Store the message (you can use a database of your choice here)
        // For simplicity, let's just print the message
        println!("Received message: {:?}", message);

        // Encrypt the message for the recipient using Vault
        let recipient = format!("{}_pubkey", message.recipient);
        let encrypted = self.vault.encrypt(&recipient, &decrypted)?;

        // Forward the encrypted message
        ctx.send(
            "websocket_transport",
            WebSocketMessage::new(encrypted, "localhost:8080".into()),
        )
        .await?;

        Ok(())
    }
}

// Define a worker for handling key exchange requests
struct KeyExchangeHandler {
    vault: Vault,
}

impl Worker for KeyExchangeHandler {
    type Message = WebSocketMessage;
    type Context = Context;

    async fn handle_message(&mut self, ctx: &mut Self::Context, msg: Self::Message) -> ockam::Result<()> {
        // Deserialize the key exchange request
        let remote_public_key: Vec<u8> = serde_json::from_slice(&msg.payload).map_err(|e| ockam::Error::new(ockam::errcode::Origin::Application, ockam::errcode::Kind::Invalid, e))?;

        // Generate a new key pair using Vault
        let key_pair = self.vault.secret_generate()?;

        // Perform key exchange (replace with your desired key exchange logic)
        let _shared_secret = self.vault.ec_diffie_hellman(&key_pair, &remote_public_key)?;

        // Send the public key to the remote node
        ctx.send(
            "websocket_transport",
            WebSocketMessage::new(key_pair, "localhost:8080".into()),
        )
        .await?;

        Ok(())
    }
}

#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    // Initialize WebSocket transport
    let ws = WebSocketTransport::create(&ctx).await?;
    ws.listen("localhost:8080").await?;

    // Create a new Vault for encryption and decryption
    let vault = Vault::create(&ctx)?;

    // Start the encrypted message handler worker
    ctx.start_worker(
        "encrypted_message_handler",
        EncryptedMessageHandler {
            vault: vault.async_try_clone().await?,
        },
    )
    .await?;

    // Start the key exchange handler worker
    ctx.start_worker(
        "key_exchange_handler",
        KeyExchangeHandler {
            vault: vault.async_try_clone().await?,
        },
    )
    .await?;

    Ok(())
}
