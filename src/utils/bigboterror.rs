use thiserror::Error;
use neo4rs;
use pyo3;
use std::io;
use std::error::Error as StdError;
use reqwest;
use kafka;
use tikv_client::Error as KVError;

use crate::bindings::spacy_bindings;

// Define a central enum for all error types in the project
#[derive(Debug, Error, Clone)]
pub enum BigbotError {
    #[error("Database operation failed: {0}")]
    DatabaseError(String),
    
    #[error("Input validation error: {0}")]
    InvalidInput(String),

    #[error("Transaction aborted")]
    TransactionAborted,

    #[error("Error interacting with external NLP library: {0}")]
    NlpError(String),

    #[error("Kafka error: {0}")]
    KafkaError(String),
    
    #[error("MQTT disconnection error")]
    MqttDisconnectionError,
    
    #[error("Neo4j error")]
    Neo4jError(#[from] neo4rs::Error),
    
    #[error("Python error")]
    PythonError(#[from] pyo3::PyErr),
    
    #[error("I/O error")]
    IoError(#[from] io::Error),
    
    #[error("Entity label conversion error: {0}")]
    EntityLabelConversionError(String),
    
    #[error("Spacy bindings error")]
    SpacyBindingsError(#[from] spacy_bindings::BigbotError),
    
    #[error("Failed to create realm: {0}")]
    RealmCreationError(String),

    #[error("Rejected error: {0}")]
    RejectedError(String),

    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Failed to create user: {0}")]
    UserCreationError(String),
    
    #[error("Failed to generate OpenID token: {0}")]
    OpenIDTokenError(#[source] reqwest::Error),
    
    #[error("Failed to authenticate user: {0}")]
    AuthenticationError(String),
    
    #[error("Failed to logout user: {0}")]
    LogoutError(#[source] reqwest::Error),
    
    #[error("Failed to filter users: {0}")]
    UserFilterError(String),
    
    #[error("Failed to create user: {0}")]
    UserCreateError(String),
    
    #[error("Failed to update user: {0}")]
    UserUpdateError(String),
    
    #[error("Failed to delete user: {0}")]
    UserDeleteError(String),
    
    #[error("Wallet not found")]
    WalletNotFound,
    
    #[error("Failed to sign credential: {0}")]
    CredentialSignError(String),
    
    #[error("Failed to verify credential: {0}")]
    CredentialVerificationError(String),
    
    #[error("Failed to get user: {0}")]
    UserGetError(String),

    #[error("An unexpected error occurred: {0}")]
    UnexpectedError(String),
}

// Implement From trait for various error types

impl From<kafka::Error> for BigbotError {
    fn from(err: kafka::Error) -> Self {
        BigbotError::KafkaError(err.to_string())
    }
}

impl From<KVError> for BigbotError {
    fn from(err: KVError) -> Self {
        BigbotError::DatabaseError(err.to_string())
    }
}

impl From<Box<dyn StdError>> for BigbotError {
    fn from(error: Box<dyn StdError>) -> Self {
        BigbotError::UnexpectedError(error.to_string())
    }
}

// Add more From implementations for other error types as needed
