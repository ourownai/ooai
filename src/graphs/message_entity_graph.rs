use crate::bindings::spacy_bindings::{Entity, EntityLabel, SPACY};
use crate::utils::bigboterror::BigbotError;
use neo4rs::{query, Graph};
use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::str::FromStr;

// Constants for Neo4j Cypher queries used in the module.
const LINK_USER_WITH_ENTITY_QUERY: &str = "\
 MERGE (u:User{user_id:$user_id}) \
 MERGE (e:Entity{label: $label, text: $text}) \
 MERGE (u)-[m:Mention{ts:$ts, score:$score}]->(e)";

const QUERY_USER_PREFERENCES_QUERY: &str = "\
 MATCH (u:User)-[m:Mention]->(e:Entity) \
 WHERE u.user_id=$id \
 RETURN e.text, e.label, m.score, m.ts";

const QUERY_ENTITY_USERS_QUERY: &str = "\
 MATCH (u:User)-[m:Mention]->(e:Entity) \
 WHERE e.text=$text AND e.label=$label \
 RETURN u.user_id, m.ts, m.score";

const QUERY_TOP_ENTITIES_QUERY: &str = "\
 MATCH (e:Entity)<-[m:Mention]-(u:User) \
 RETURN e.text, e.label, COUNT(m) as count \
 ORDER BY count DESC \
 LIMIT $limit";

// Handler for managing user preferences within a Neo4j graph database.
pub struct PreferenceGraphHandler {
    neo_client: Arc<Graph>,
}

// Represents a user preference, including the user ID, related entity, timestamp, and preference score.
pub struct UserPreference {
    user_id: i64,
    entity: Entity,
    ts: i64,
    score: f64,
}

impl PreferenceGraphHandler {
    // Constructs a new handler with the given Neo4j graph client.
    pub fn new(neo_client: Arc<Graph>) -> Self {
        Self { neo_client }
    }

    // Processes a user's utterance to extract entities, analyze sentiment, and store them as preferences.
    pub async fn handle(
        &self,
        user_id: i64,
        utterance: &str,
        timestamp: i64,
    ) -> Result<(), BigbotError> {
        let (entities, sentiment) = extract_entities_and_sentiment(utterance).await?;
        for entity in entities {
            self.neo_client
                .run(
                    query(LINK_USER_WITH_ENTITY_QUERY)
                        .param("user_id", user_id)
                        .param("ts", timestamp)
                        .param("label", entity.label.to_string())
                        .param("text", entity.text.clone())
                        .param("score", sentiment),
                )
                .await
                .map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
        }
        Ok(())
    }

    // Queries the database for a user's preferences and returns them.
    pub async fn query_user_preferences(
        &self,
        user_id: i64,
    ) -> Result<Vec<UserPreference>, BigbotError> {
        let mut results = self
            .neo_client
            .execute(query(QUERY_USER_PREFERENCES_QUERY).param("id", user_id))
            .await
            .map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
        let mut preferences = Vec::new();
        while let Some(row) = results.next().await.map_err(BigbotError::Neo4jError)? {
            let ts: i64 = row.get("m.ts").map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
            let text: String = row.get("e.text").map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
            let label: String = row.get("e.label").map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
            let score: f64 = row.get("m.score").map_err(|e| BigbotError::DatabaseError(e.to_string()))?;

            let entity_label = EntityLabel::from_str(&label).map_err(|_| {
                BigbotError::EntityLabelConversionError(format!("Invalid entity label: {}", label))
            })?;

            preferences.push(UserPreference {
                user_id,
                entity: Entity {
                    text,
                    label: entity_label,
                },
                ts,
                score,
            });
        }
        Ok(preferences)
    }

    // Finds users interested in a specific entity based on their preferences.
    pub async fn query_entity_users(
        &self,
        entity: &Entity,
    ) -> Result<Vec<UserPreference>, BigbotError> {
        let mut results = self
            .neo_client
            .execute(
                query(QUERY_ENTITY_USERS_QUERY)
                    .param("text", entity.text.clone())
                    .param("label", entity.label.to_string()),
            )
            .await
            .map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
        let mut users = Vec::new();
        while let Some(row) = results.next().await.map_err(BigbotError::Neo4jError)? {
            let ts: i64 = row.get("m.ts").map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
            let user_id: i64 = row.get("u.user_id").map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
            let score: f64 = row.get("m.score").map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
            users.push(UserPreference {
                entity: entity.clone(),
                ts,
                score,
                user_id,
            });
        }
        Ok(users)
    }

    // Retrieves the top N entities based on the number of user mentions.
    pub async fn query_top_entities(
        &self,
        limit: i64,
    ) -> Result<Vec<(String, String, i64)>, BigbotError> {
        let mut results = self
            .neo_client
            .execute(query(QUERY_TOP_ENTITIES_QUERY).param("limit", limit))
            .await
            .map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
        let mut top_entities = Vec::new();
        while let Some(row) = results.next().await.map_err(BigbotError::Neo4jError)? {
            let text: String = row.get("e.text").map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
            let label: String = row.get("e.label").map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
            let count: i64 = row.get("count").map_err(|e| BigbotError::DatabaseError(e.to_string()))?;
            top_entities.push((text, label, count));
        }
        Ok(top_entities)
    }

    // Computes the similarity between two users based on their preferences.
    pub async fn compute_user_similarity(
        &self,
        user_id1: i64,
        user_id2: i64,
    ) -> Result<f64, BigbotError> {
        let prefs1 = self.query_user_preferences(user_id1).await?;
        let prefs2 = self.query_user_preferences(user_id2).await?;
        let prefs1_map: HashMap<String, f64> = prefs1
            .into_iter()
            .map(|p| (p.entity.text, p.score))
            .collect();
        let prefs2_map: HashMap<String, f64> = prefs2
            .into_iter()
            .map(|p| (p.entity.text, p.score))
            .collect();
        let mut dot_product = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;
        for (entity, score1) in &prefs1_map {
            if let Some(score2) = prefs2_map.get(entity) {
                dot_product += score1 * score2;
            }
            norm1 += score1 * score1;
        }
        for (_, score2) in &prefs2_map {
            norm2 += score2 * score2;
        }
        let similarity = if norm1 > 0.0 && norm2 > 0.0 {
            dot_product / (norm1.sqrt() * norm2.sqrt())
        } else {
            0.0
        };
        Ok(similarity)
    }
}

// Extracts entities and analyzes sentiment from a given utterance using the SPACY library.
async fn extract_entities_and_sentiment(utterance: &str) -> Result<(Vec<Entity>, f64), BigbotError> {
    let doc = SPACY
        .model_default()
        .nlp(utterance.to_string())
        .await
        .map_err(|e| BigbotError::NlpError(e.to_string()))?;
    let entities = Python::with_gil(|py| {
        pyo3::types::PyList::new(py, doc.ents(py)).extract::<Vec<Entity>>().map_err(|e| BigbotError::NlpError(e.to_string()))
    })?;

    let sentiment = Python::with_gil(|py| {
        doc.sentiment(py).map_err(BigbotError::from)
    })?;

    Ok((entities, sentiment))
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_handle_user_utterance() {
        let graph = Arc::new(Graph::new("localhost:7687", "neo4j", "password").await.unwrap());
        let handler = PreferenceGraphHandler::new(graph);
        // Example test - adjust to your testing setup
        let user_id = 1;
        let utterance = "I love visiting museums and art galleries.";
        let timestamp = 1625097600; // Example timestamp
        let result = handler.handle(user_id, utterance, timestamp).await;
        assert!(result.is_ok(), "Handling user utterance failed");
        // Further assertions to verify the correctness of the database state...
    }

    // Additional tests for `query_user_preferences`, `query_entity_users`, `query_top_entities`, and `compute_user_similarity`...
}