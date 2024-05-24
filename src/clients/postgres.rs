use crate::clients::kv::KVStore;
use crate::utils::bigboterror::BigbotError;

use async_trait::async_trait;
use std::sync::Arc;
use std::sync::OnceLock;
use thiserror::Error;
use tokio_postgres::{Client, NoTls};

pub static PG_CLIENT: OnceLock<Arc<Client>> = OnceLock::new();

// Define a custom error type for PostgreSQL-related errors
#[derive(Error, Debug)]
pub enum PostgresError {
    #[error("Failed to connect to PostgreSQL: {0}")]
    ConnectionError(#[source] tokio_postgres::Error),
    #[error("Failed to execute PostgreSQL query: {0}")]
    QueryError(#[source] tokio_postgres::Error),
}

// Create a new PostgreSQL client instance
pub async fn new_postgres_client(params: &str) -> Result<Arc<Client>, PostgresError> {
    let cfg = params.parse::<tokio_postgres::Config>().unwrap();
    let (client, conn) = cfg.connect(NoTls).await.map_err(PostgresError::ConnectionError)?;
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection with pg is broken: {}", e);
        }
    });
    Ok(Arc::new(client))
}

// Initialize the PostgreSQL client from environment variables
pub async fn init_postgres_client_from_env() -> Result<(), PostgresError> {
    let addr = std::env::var("POSTGRES_ADDR").unwrap_or_else(|_| "127.0.0.1:5432".to_string());
    let user = std::env::var("POSTGRES_USER").unwrap_or_default();
    let uri = format!("postgresql://{}@{}/postgres?keepalives=1", user, addr);
    let _ = PG_CLIENT.set(new_postgres_client(&uri).await?);
    Ok(())
}

// Define a struct for the PostgreSQL table-based key-value client
pub struct PGTableKVClient {
    pg_client: Arc<Client>,
    table_name: String,
    key_name: String,
    val_name: String,
}

impl PGTableKVClient {
    // Constructor for PGTableKVClient
    pub fn new(
        table_name: String,
        pg_client: Arc<Client>,
        key_name: String,
        val_name: String,
    ) -> Self {
        Self {
            table_name,
            pg_client,
            key_name,
            val_name,
        }
    }

    // Retrieve all key-value pairs from the PostgreSQL table
    pub async fn kvs(&self) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PostgresError> {
        let sql = format!(
            "SELECT {},{} FROM {}",
            self.key_name, self.val_name, self.table_name
        );
        let rows = self.pg_client.query(&sql, &[]).await.map_err(PostgresError::QueryError)?;
        Ok(rows.into_iter().map(|x| (x.get(0), x.get(1))).collect())
    }

    // Retrieve the value associated with a key from the PostgreSQL table
    async fn get_value(&self, key: &[u8]) -> Result<Option<Vec<u8>>, PostgresError> {
        let sql = format!(
            "SELECT {} FROM {} WHERE {}=$1",
            self.val_name, self.table_name, self.key_name
        );
        let result = self.pg_client.query_opt(&sql, &[&key]).await.map_err(PostgresError::QueryError)?;
        Ok(result.map(|row| row.get(0)))
    }

    // Set a key-value pair in the PostgreSQL table
    async fn set_value(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), PostgresError> {
        let sql = format!(
            "INSERT INTO {} ({},{}) VALUES ($1, $2) ON CONFLICT ({}) DO UPDATE SET {}=$2",
            self.table_name, self.key_name, self.val_name, self.key_name, self.val_name
        );
        self.pg_client.execute(&sql, &[&key, &value]).await.map_err(PostgresError::QueryError)?;
        Ok(())
    }

    // Delete a key-value pair from the PostgreSQL table
    async fn delete_value(&self, key: &[u8]) -> Result<(), PostgresError> {
        let sql = format!("DELETE FROM {} WHERE {}=$1", self.table_name, self.key_name);
        self.pg_client.execute(&sql, &[&key]).await.map_err(PostgresError::QueryError)?;
        Ok(())
    }
}

#[async_trait]
impl KVStore for PGTableKVClient {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, BigbotError> {
        self.get_value(key)
            .await
            .map_err(|e| BigbotError::DatabaseError(e.into()))
    }

    async fn set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), BigbotError> {
        self.set_value(key, value)
            .await
            .map_err(|e| BigbotError::DatabaseError(e.into()))
    }

    async fn delete(&self, key: &[u8]) -> Result<(), BigbotError> {
        self.delete_value(key)
            .await
            .map_err(|e| BigbotError::DatabaseError(e.into()))
    }

    async fn keys(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>, BigbotError> {
        Ok(self
            .kvs()
            .await
            .map_err(|e| BigbotError::DatabaseError(e.into()))?
            .into_iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(k, _)| k)
            .collect())
    }
}
