use neo4rs::{Graph, query};
use std::sync::Arc;
use std::convert::TryFrom;
use thiserror::Error;
use std::collections::HashMap;
use crate::utils::bigboterror::BigbotError;


// Define custom error types for the identity graph operations
#[derive(Error, Debug)]
pub enum IdentityGraphError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] BigbotError), // Convert from BigbotError to IdentityGraphError
    #[error("Invalid identity type: {0}")]
    InvalidIdentityType(String), // Error for invalid identity type
    #[error("Invalid user ID: {0}")]
    InvalidUserId(i64), // Error for invalid user ID
}

// Struct representing an Identity
#[derive(Clone, Debug)]
pub struct Identity {
    user_id: i64, // User ID associated with the identity
    identity_type: IdentityType, // Type of the identity
    ts: i64, // Timestamp of the identity creation or update
}

// Enum representing different types of identities
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum IdentityType {
    PhoneNumber(String),
    PhoneIdentifier(String),
    Email(String),
    UserAgent(String),
    IP(String),
}

// Implement methods for IdentityType to facilitate operations
impl IdentityType {
    // Method to return the identity type as a tuple of its label and value
    pub fn as_tuple(&self) -> (&str, &String) {
        match self {
            IdentityType::PhoneNumber(v) => ("PhoneNumber", v),
            IdentityType::PhoneIdentifier(v) => ("PhoneIdentifier", v),
            IdentityType::Email(v) => ("Email", v),
            IdentityType::UserAgent(v) => ("UserAgent", v),
            IdentityType::IP(v) => ("IP", v),
        }
    }
}

pub struct IdentityGraph {
    nodes: HashMap<i64, IdentityNode>,
    edges: Vec<IdentityEdge>,
}

pub struct IdentityNode {
    pub user_id: i64,
    pub attributes: HashMap<String, String>,
}

pub struct IdentityEdge {
    pub from: i64,
    pub to: i64,
    pub relationship: String,
}

impl IdentityGraph {
    pub fn new() -> Self {
        IdentityGraph {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: IdentityNode) {
        self.nodes.insert(node.user_id, node);
    }

    pub fn add_edge(&mut self, edge: IdentityEdge) {
        self.edges.push(edge);
    }

    pub fn get_node(&self, user_id: i64) -> Option<&IdentityNode> {
        self.nodes.get(&user_id)
    }

    pub fn get_neighbors(&self, user_id: i64) -> Vec<&IdentityNode> {
        let mut neighbors = Vec::new();
        for edge in &self.edges {
            if edge.from == user_id {
                if let Some(node) = self.nodes.get(&edge.to) {
                    neighbors.push(node);
                }
            } else if edge.to == user_id {
                if let Some(node) = self.nodes.get(&edge.from) {
                    neighbors.push(node);
                }
            }
        }
        neighbors
    }

    pub fn get_identity_nodes(&self, identity_type: &IdentityType) -> Vec<&IdentityNode> {
        let (identity_label, identity_value) = identity_type.as_tuple();
        self.nodes.values().filter(|node| {
            node.attributes.get(identity_label) == Some(identity_value)
        }).collect()
    }
}

// Implement TryFrom for converting a (String, String) tuple into an IdentityType
impl TryFrom<(String, String)> for IdentityType {
    type Error = IdentityGraphError;

    fn try_from(value: (String, String)) -> Result<Self, Self::Error> {
        match value.0.as_str() {
            "phone_number" => Ok(IdentityType::PhoneNumber(value.1)),
            "phone_identifier" => Ok(IdentityType::PhoneIdentifier(value.1)),
            "email" => Ok(IdentityType::Email(value.1)),
            "user_agent" => Ok(IdentityType::UserAgent(value.1)),
            "ip" => Ok(IdentityType::IP(value.1)),
            _ => Err(IdentityGraphError::InvalidIdentityType(value.0)), // Handle invalid identity types
        }
    }
}

// Handler for interacting with the identity graph in the database
pub struct IdentityGraphHandler {
    client: Arc<Graph>, // Client for Neo4j database operations
}

// Implementation of methods for IdentityGraphHandler
impl IdentityGraphHandler {
    // Constructor to create a new instance of the handler
    pub fn new(client: Arc<Graph>) -> Self {
        Self { client }
    }

    // Method to add or update an identity in the graph database
    pub async fn add_identity(&self, user_id: i64, identity_type: IdentityType, ts: i64) -> Result<(), IdentityGraphError> {
        if user_id <= 0 {
            return Err(IdentityGraphError::InvalidUserId(user_id));
        }

        let (identity_label, identity_value) = identity_type.as_tuple(); // Decompose the identity type
        // Construct the Cypher query for adding/updating the identity
        let query_str = format!(
            "MERGE (u:User {{user_id: $user_id}})
            MERGE (i:Identity {{type: '{identity_label}', identifier: $identity_value}})
            MERGE (u)-[:Has]->(i) SET i.ts = $ts",
            identity_label = identity_label,
        );
        let query = query(&query_str)
            .param("user_id", user_id)
            .param("identity_value", identity_value.to_string())
            .param("ts", ts);

        // Execute the query and handle potential errors
        self.client.run(query).await.map_err(IdentityGraphError::DatabaseError)?;
        Ok(()) // Return Ok if operation is successful
    }

    // Method to retrieve identities for a user
    pub async fn get_user_identities(&self, user_id: i64) -> Result<Vec<Identity>, IdentityGraphError> {
        if user_id <= 0 {
            return Err(IdentityGraphError::InvalidUserId(user_id));
        }

        // Construct the Cypher query for retrieving user identities
        let query_str = "
            MATCH (u:User {user_id: $user_id})-[:Has]->(i:Identity)
            RETURN i.type AS identity_type, i.identifier AS identity_value, i.ts AS ts
        ";
        let query = query(query_str)
            .param("user_id", user_id);

        // Execute the query and handle potential errors
        let result = self.client.execute(query).await.map_err(IdentityGraphError::DatabaseError)?;

        // Process the query result and convert it into Identity structs
        let identities = result.rows().map(|row| {
            let identity_type: String = row.get("identity_type").unwrap();
            let identity_value: String = row.get("identity_value").unwrap();
            let ts: i64 = row.get("ts").unwrap();
            let identity_type = IdentityType::try_from((identity_type, identity_value)).unwrap();
            Identity { user_id, identity_type, ts }
        }).collect();

        Ok(identities)
    }
}

// The test module and additional implementation details would follow, maintaining consistent error handling and code structure.
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    async fn new_neo4j_client_from_env() -> Arc<Graph> {
        Arc::new(Graph::new("neo4j://localhost:7687", "username", "password").await.unwrap())
    }

    #[tokio::test]
    async fn test_identity_graph_handler() {
        let client = new_neo4j_client_from_env().await;
        let handler = IdentityGraphHandler::new(client);

        let user_id = 1;
        let phone_number = IdentityType::PhoneNumber("1234567890".to_string());
        let email = IdentityType::Email("user@example.com".to_string());

        // Add phone number identity
        let result = handler.add_identity(user_id, phone_number.clone(), 1627846874).await;
        assert!(result.is_ok());

        // Add email identity
        let result = handler.add_identity(user_id, email.clone(), 1627846875).await;
        assert!(result.is_ok());

        // Retrieve user identities
        let identities = handler.get_user_identities(user_id).await.unwrap();
        assert_eq!(identities.len(), 2);

        let identity_types: HashSet<IdentityType> = identities.into_iter().map(|identity| identity.identity_type).collect();
        assert!(identity_types.contains(&phone_number));
        assert!(identity_types.contains(&email));

        // Test invalid user ID
        let invalid_user_id = -1;
        let result = handler.add_identity(invalid_user_id, phone_number.clone(), 1627846876).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IdentityGraphError::InvalidUserId(id) if id == invalid_user_id));
    }
}