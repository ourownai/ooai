use crate::iam::user::User;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::fmt;

/// Represents a group of users.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Group {
    /// The unique identifier of the group.
    id: String,
    /// The name of the group.
    name: String,
    /// An optional description of the group.
    description: Option<String>,
    /// The list of users in the group.
    users: Vec<User>,
    /// The timestamp when the group was created.
    created_at: DateTime<Utc>,
    /// The timestamp when the group was last modified.
    last_modified_at: DateTime<Utc>,
    /// The maximum number of users allowed in the group.
    max_users: Option<usize>,
}

/// Represents errors that can occur when interacting with a group.
#[derive(Error, Debug)]
pub enum GroupError {
    #[error("user not found")]
    UserNotFound,
    #[error("user already exists in the group")]
    UserAlreadyExists,
    #[error("group is full")]
    GroupFull,
}

impl Group {
    /// Creates a new group with the given id, name, and maximum number of users.
    pub fn new(id: String, name: String, max_users: Option<usize>) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description: None,
            users: Vec::new(),
            created_at: now,
            last_modified_at: now,
            max_users,
        }
    }

    /// Sets the description of the group.
    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
        self.last_modified_at = Utc::now();
    }

    /// Adds a user to the group.
    /// Returns an error if the user already exists or the group is full.
    pub fn add_user(&mut self, user: User) -> Result<(), GroupError> {
        if let Some(max) = self.max_users {
            if self.users.len() >= max {
                return Err(GroupError::GroupFull);
            }
        }
        if self.users.iter().any(|u| u.id == user.id) {
            return Err(GroupError::UserAlreadyExists);
        }
        self.users.push(user);
        self.last_modified_at = Utc::now();
        Ok(())
    }

    /// Removes a user from the group based on the user's id.
    /// Returns the removed user if found, otherwise returns an error.
    pub fn remove_user(&mut self, user_id: &str) -> Result<User, GroupError> {
        if let Some(index) = self.users.iter().position(|user| user.id == user_id) {
            self.last_modified_at = Utc::now();
            Ok(self.users.remove(index))
        } else {
            Err(GroupError::UserNotFound)
        }
    }

    /// Finds a user in the group based on the user's id.
    /// Returns a reference to the user if found, otherwise returns None.
    pub fn find_user(&self, user_id: &str) -> Option<&User> {
        self.users.iter().find(|user| user.id == user_id)
    }

    // Additional functionality implementations would go here.
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Group [{}]: {}, Users: {}", self.id, self.name, self.users.len())
    }
}