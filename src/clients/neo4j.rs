use std::sync::Arc;

use neo4rs::Graph;

use crate::utils::bigboterror::BigbotError;

pub async fn new_neo4j_client(
    uri: &str,
    user: &str,
    passwd: &str,
) -> Result<Arc<Graph>, BigbotError> {
    Ok(Arc::new(Graph::new(uri, user, passwd).await?))
}

pub async fn new_neo4j_client_from_env() -> Result<Arc<Graph>, BigbotError> {
    let uri = std::env::var("NEO4J_URI").unwrap_or_default();
    let user = std::env::var("NEO4J_USER").unwrap_or_default();
    let passwd = std::env::var("NEO4J_PASSWD").unwrap_or_default();
    println!("uri: {}, user: {}, passwd: {}", uri, user, passwd);
    new_neo4j_client(&uri, &user, &passwd).await
}
