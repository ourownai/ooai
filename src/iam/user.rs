//! # User Management
//!
//! This module provides a comprehensive solution for managing user entities within a system. It includes functionality for creating, updating, deleting, and retrieving user records, along with managing their associated data like wallets and verifiable credentials. The system employs a Merkle tree structure to securely and efficiently verify user credentials. Key features include:
//!
//! - **User Creation and Management**: Facilitates the creation of user profiles, including their identification, authentication details, and associated cryptographic wallets. Users can update their profile information (username, email, and wallet) and manage their verifiable credentials.
//! - **Verifiable Credentials**: Allows users to add credentials to their profile and generate proofs for these credentials, leveraging the Merkle tree structure for secure and efficient verification.
//! - **Error Handling**: Defines custom errors for common user-related operations, improving the robustness and reliability of the system.
//! - **Extensibility**: Through the use of builder patterns and service-oriented architecture, the module is designed for easy expansion and integration into larger systems.
//!
//! The module is structured around several key components, including the `User` struct for representing user profiles, the `UserError` enum for error management, and the `UserService` class for handling business logic related to user operations.


use crate::iam::merkle_tree::MerkleTree;
use crate::iam::verifiable_credentials::{VerifiableCredential, VCBuilder};
use crate::iam::wallet::Wallet;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use std::fmt;

/// Represents a user in the system.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    /// The unique identifier of the user.
    pub id: String,
    /// The username of the user.
    pub username: String,
    /// The email address of the user.
    pub email: String,
    /// The user's wallet.
    pub wallet: Wallet,
    /// The user's verifiable credentials.
    pub credentials: HashMap<String, VerifiableCredential>,
    /// The user's Merkle tree for credential proofs.
    pub credential_tree: MerkleTree,
    /// The timestamp when the user was created.
    pub created_at: DateTime<Utc>,
    /// The timestamp when the user was last modified.
    pub last_modified_at: DateTime<Utc>,
}

/// Represents errors that can occur when interacting with a user.
#[derive(Error, Debug)]
pub enum UserError {
    #[error("user not found")]
    UserNotFound,
    #[error("failed to create user: {0}")]
    UserCreationError(String),
    #[error("failed to update user: {0}")]
    UserUpdateError(String),
    #[error("failed to delete user: {0}")]
    UserDeleteError(String),
}

impl User {
    /// Creates a new user with the given id, username, email, and wallet.
    pub fn new(id: String, username: String, email: String, wallet: Wallet) -> Self {
        let now = Utc::now();
        Self {
            id,
            username,
            email,
            wallet,
            credentials: HashMap::new(),
            credential_tree: MerkleTree::new(),
            created_at: now,
            last_modified_at: now,
        }
    }

    /// Updates the user's username.
    pub fn set_username(&mut self, username: String) {
        self.username = username;
        self.last_modified_at = Utc::now();
    }

    /// Updates the user's email address.
    pub fn set_email(&mut self, email: String) {
        self.email = email;
        self.last_modified_at = Utc::now();
    }

    /// Updates the user's wallet.
    pub fn set_wallet(&mut self, wallet: Wallet) {
        self.wallet = wallet;
        self.last_modified_at = Utc::now();
    }

    /// Adds a new verifiable credential to the user.
    pub fn add_credential(&mut self, credential: VerifiableCredential) {
        self.credentials.insert(credential.id.clone(), credential);
        self.credential_tree.update(&credential.id.as_bytes());
        self.last_modified_at = Utc::now();
    }

    /// Retrieves a verifiable credential by its ID.
    pub fn get_credential(&self, credential_id: &str) -> Option<&VerifiableCredential> {
        self.credentials.get(credential_id)
    }

    /// Generates a proof for a specific credential.
    pub fn generate_credential_proof(&self, credential_id: &str) -> Option<String> {
        let credential = self.get_credential(credential_id)?;
        let proof = self.credential_tree.get_proof(&credential.id.as_bytes());
        Some(serde_json::to_string(&proof).ok()?)
    }

    // Additional functionality implementations would go here.
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User [{}]: {}", self.id, self.username)
    }
}

/// Represents a builder for creating a user.
pub struct UserBuilder {
    id: String,
    username: String,
    email: String,
    wallet: Wallet,
}

impl UserBuilder {
    /// Creates a new user builder.
    pub fn new(id: String) -> Self {
        Self {
            id,
            username: String::new(),
            email: String::new(),
            wallet: Wallet::default(),
        }
    }

    /// Sets the username for the user.
    pub fn username(mut self, username: String) -> Self {
        self.username = username;
        self
    }

    /// Sets the email address for the user.
    pub fn email(mut self, email: String) -> Self {
        self.email = email;
        self
    }

    /// Sets the wallet for the user.
    pub fn wallet(mut self, wallet: Wallet) -> Self {
        self.wallet = wallet;
        self
    }

    /// Builds and returns the user.
    pub fn build(self) -> User {
        User::new(self.id, self.username, self.email, self.wallet)
    }
}

/// Represents a service for managing users.
pub struct UserService {
    users: HashMap<String, User>,
}

impl UserService {
    /// Creates a new user service.
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    /// Creates a new user with the given details.
    pub fn create_user(&mut self, user: User) -> Result<&User, UserError> {
        if self.users.contains_key(&user.id) {
            return Err(UserError::UserCreationError(format!("User with ID '{}' already exists", user.id)));
        }
        self.users.insert(user.id.clone(), user);
        Ok(self.users.get(&user.id).unwrap())
    }

    /// Retrieves a user by their ID.
    pub fn get_user(&self, user_id: &str) -> Result<&User, UserError> {
        self.users.get(user_id).ok_or(UserError::UserNotFound)
    }

    /// Updates an existing user.
    pub fn update_user(&mut self, user: User) -> Result<&User, UserError> {
        if !self.users.contains_key(&user.id) {
            return Err(UserError::UserUpdateError(format!("User with ID '{}' does not exist", user.id)));
        }
        self.users.insert(user.id.clone(), user);
        Ok(self.users.get(&user.id).unwrap())
    }

    /// Deletes a user by their ID.
    pub fn delete_user(&mut self, user_id: &str) -> Result<(), UserError> {
        if !self.users.contains_key(user_id) {
            return Err(UserError::UserDeleteError(format!("User with ID '{}' does not exist", user_id)));
        }
        self.users.remove(user_id);
        Ok(())
    }

    // Additional service methods would go here.
}