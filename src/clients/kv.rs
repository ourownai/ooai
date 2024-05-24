use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::utils::bigboterror::BigbotError;
use thiserror::Error;

// Define the custom error type using thiserror
#[derive(Error, Debug)]
pub enum KVError {
    #[error("Key not found")]
    KeyNotFound,
    #[error("Failed to perform operation")]
    OperationFailed,
}

impl From<KVError> for BigbotError {
    fn from(err: KVError) -> Self {
        match err {
            KVError::KeyNotFound => BigbotError::KeyNotFound,
            KVError::OperationFailed => BigbotError::OperationFailed,
        }
    }
}

// Define the KVStore trait with async methods
#[async_trait]
pub trait KVStore: Send + Sync {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, BigbotError>;
    async fn set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), BigbotError>;
    async fn delete(&self, key: &[u8]) -> Result<(), BigbotError>;
    async fn keys(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>, BigbotError>;
}

// Implement the KVStore trait for Arc<dyn KVStore>
#[async_trait]
impl KVStore for Arc<dyn KVStore> {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, BigbotError> {
        self.as_ref().get(key).await
    }

    async fn set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), BigbotError> {
        self.as_ref().set(key, value).await
    }

    async fn delete(&self, key: &[u8]) -> Result<(), BigbotError> {
        self.as_ref().delete(key).await
    }

    async fn keys(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>, BigbotError> {
        self.as_ref().keys(prefix).await
    }
}

// Define the PrefixedKVStore struct
pub struct PrefixedKVStore<T> {
    store: T,
    prefix: Vec<u8>,
}

impl<T> PrefixedKVStore<T> {
    // Constructor for PrefixedKVStore
    pub fn new(store: T, prefix: Vec<u8>) -> PrefixedKVStore<T> {
        Self { store, prefix }
    }

    // Helper method to create a prefixed key
    fn make_prefix(&self, key: &[u8]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.prefix.len() + key.len());
        buf.extend_from_slice(self.prefix.as_slice());
        buf.extend_from_slice(key);
        buf
    }
}

// Implement the KVStore trait for PrefixedKVStore<T>
#[async_trait]
impl<T: KVStore> KVStore for PrefixedKVStore<T> {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, BigbotError> {
        self.store.get(self.make_prefix(key).as_slice()).await
    }

    async fn set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), BigbotError> {
        self.store
            .set(self.make_prefix(key.as_slice()), value)
            .await
    }

    async fn delete(&self, key: &[u8]) -> Result<(), BigbotError> {
        self.store.delete(self.make_prefix(key).as_slice()).await
    }

    async fn keys(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>, BigbotError> {
        self.store.keys(self.make_prefix(prefix).as_slice()).await
    }
}

// Define the MemoryKVStore struct for testing purposes
#[derive(Default)]
pub struct MemoryKVStore {
    values: Arc<Mutex<BTreeMap<Vec<u8>, Vec<u8>>>>,
}

// Implement the KVStore trait for MemoryKVStore
#[async_trait]
impl KVStore for MemoryKVStore {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, BigbotError> {
        Ok(self.values.lock().await.get(key).cloned())
    }

    async fn set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), BigbotError> {
        self.values.lock().await.insert(key, value);
        Ok(())
    }

    async fn delete(&self, key: &[u8]) -> Result<(), BigbotError> {
        self.values.lock().await.remove(key);
        Ok(())
    }

    async fn keys(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>, BigbotError> {
        Ok(self
            .values
            .lock()
            .await
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(k, _)| k.clone())
            .collect())
    }
}