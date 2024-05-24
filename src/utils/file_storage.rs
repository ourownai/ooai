use std::io::{self, Read};
use std::time::{Duration, SystemTime};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use chrono::{DateTime, Utc};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use bytes::Bytes;
use iroh::client::Iroh;

// Error handling
#[derive(Error, Debug)]
pub enum FileStorageError {
    #[error("File error: {0}")]
    FileError(#[from] io::Error),
    #[error("Iroh error")]
    IrohError,
    #[error("Encryption error")]
    EncryptionError,
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Environment error: {0}")]
    EnvironmentError(#[from] std::env::VarError),
}


// Assuming `UploadedFile` and `handle_upload_file` are part of an async context
#[derive(Serialize, Deserialize)]
pub struct UploadedFile {
    pub user_id: String,
    pub data: Vec<u8>,
}

impl UploadedFile {
    // Additional functionality: Async handling of file upload
    pub async fn handle_upload(
        self,
        secret_store_url: &str,
        identity_graph_url: &str,
    ) -> Result<String, FileStorageError> {
        let key = get_key(&self.user_id, secret_store_url, identity_graph_url).await?;
        let encrypted_data = encrypt_data(&self.data, &key)?;

        let iroh = Iroh::default();
        let cid = iroh.add(Bytes::from(encrypted_data)).await.map_err(|_| FileStorageError::IrohError)?;
        let cid_string = cid.to_string();

        let expiry_time = calculate_expiry_time(SystemTime::now(), Duration::from_secs(3600));
        store_file_metadata(&cid_string, &self.user_id, &expiry_time)?;

        Ok(cid_string)
    }

    // Additional functionality: Async retrieval of file data
    pub async fn retrieve_file(
        cid_string: &str,
        user_id: &str,
        secret_store_url: &str,
        identity_graph_url: &str,
    ) -> Result<Vec<u8>, FileStorageError> {
        let key = get_key(user_id, secret_store_url, identity_graph_url).await?;

        let iroh = Iroh::default();
        let file = iroh.get(cid_string).await.map_err(|_| FileStorageError::IrohError)?;
        let encrypted_data = file.to_vec();

        decrypt_data(&encrypted_data, &key)
    }
}

// Modular function to get encryption key
async fn get_key(
    user_id: &str,
    secret_store_url: &str,
    identity_graph_url: &str,
) -> Result<Vec<u8>, FileStorageError> {
    let key_hash = get_key_hash(user_id, identity_graph_url).await?;
    get_encryption_key(&key_hash, secret_store_url).await
}

// Placeholder for async HTTP request to get the key hash
async fn get_key_hash(user_id: &str, _identity_graph_url: &str) -> Result<String, FileStorageError> {
    // Simulate API call
    Ok(generate_random_string(32))
}

// Placeholder for async HTTP request to get the encryption key
async fn get_encryption_key(key_hash: &str, _secret_store_url: &str) -> Result<Vec<u8>, FileStorageError> {
    // Simulate API call
    Ok(generate_random_bytes(32))
}

// Encryption utility function
pub fn encrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>, FileStorageError> {
    let cipher = Aes256Gcm::new_from_slice(key).expect("Invalid key size");
    let nonce = Nonce::from_slice(&generate_random_bytes(12));
    cipher
        .encrypt(nonce, data)
        .map_err(|_| FileStorageError::EncryptionError)
}

// Decryption utility function
fn decrypt_data(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, FileStorageError> {
    let cipher = Aes256Gcm::new_from_slice(key).expect("Invalid key size");
    let nonce = Nonce::from_slice(&encrypted_data[..12]);
    let ciphertext = &encrypted_data[12..];
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| FileStorageError::EncryptionError)
}

// Utility functions for generating random values
fn generate_random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn generate_random_bytes(len: usize) -> Vec<u8> {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Standard)
        .take(len)
        .collect()
}

// Placeholder for storing file metadata
fn store_file_metadata(cid: &str, user_id: &str, expiry_time: &DateTime<Utc>) -> Result<(), FileStorageError> {
    // Simulate storing metadata
    Ok(())
}

// Placeholder for calculating expiry time
fn calculate_expiry_time(current_time: SystemTime, duration: Duration) -> DateTime<Utc> {
    // Simulate calculating expiry time
    DateTime::<Utc>::from(current_time + duration)
}
