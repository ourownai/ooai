use crate::bindings::spacy_bindings::Entity;
use crate::utils::bigboterror::BigbotError;
use crate::event::Event;
use crate::event::Location;

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use neo4rs::{Graph, query};
use std::sync::Arc;
use thiserror::Error;


#[derive(Debug, Copy, Clone)]
pub struct Duration(u64, u64);

#[derive(Error, Debug)]
pub enum EventHandlerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] neo4rs::Error),
    #[error("Application error: {0}")]
    ApplicationError(#[from] BigbotError),
    #[error("Invalid location format")]
    InvalidLocationFormat,
    #[error("Location must have exactly three components")]
    InvalidLocationComponents,
}

pub struct EventHandler {
    graph_client: Arc<Graph>,
}

impl Event {
    pub fn new(
        id: i64,
        unique_id: String,
        user_id: String,
        time: DateTime<Utc>,
        header: String,
        name: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        attributes: HashMap<String, String>,
        event_type: Option<Entity>,
        location: Location,
        significance: f64,
        duration: Duration,
        dependencies: Vec<Arc<Event>>,
        start: i64,
        end: i64,
        resource: String,
        tags: Vec<String>,
    ) -> Self {
        Self {
            id,
            unique_id,
            user_id,
            time,
            header,
            name,
            start_time,
            end_time,
            attributes,
            event_type,
            location,
            significance,
            duration,
            dependencies,
            start,
            end,
            resource,
            tags,
        }
    }

    // Getter methods for Event properties
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn significance(&self) -> f64 {
        self.significance
    }

    pub fn is_schedulable(&self) -> bool {
        self.dependencies.iter().all(|dep| dep.duration.1 < self.duration.0)
    }
}

impl Location {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(x, y, z)
    }

    pub fn distance(&self, other: &Self) -> f32 {
        (((self.0 - other.0).powi(2) + (self.1 - other.1).powi(2) + (self.2 - other.2).powi(2)) as f32).sqrt()
    }
}

impl From<(f32, f32, f32)> for Location {
    fn from(coords: (f32, f32, f32)) -> Self {
        Self::new(coords.0, coords.1, coords.2)
    }
}

impl TryFrom<String> for Location {
    type Error = EventHandlerError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<f32> = value
            .split(',')
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| EventHandlerError::InvalidLocationFormat)?;

        if parts.len() == 3 {
            Ok(Self::new(parts[0], parts[1], parts[2]))
        } else {
            Err(EventHandlerError::InvalidLocationComponents)
        }
    }
}

impl From<Location> for String {
    fn from(location: Location) -> Self {
        format!("{},{},{}", location.0, location.1, location.2)
    }
}

impl EventHandler {
    pub fn new(graph_client: Arc<Graph>) -> Self {
        Self { graph_client }
    }

    pub async fn add_new_event(&self, event: &Event) -> Result<(), EventHandlerError> {
        self.create_event_node(event).await?;
        for dependency in &event.dependencies {
            self.create_event_node(dependency).await?;
            self.link_events(event.id, dependency.id()).await?;
        }
        if let Some(event_type) = &event.event_type {
            self.link_event_to_entity(event, event_type).await?;
        }
        Ok(())
    }

    async fn create_event_node(&self, event: &Event) -> Result<(), EventHandlerError> {
        const QUERY: &str = "MERGE (e:Event {id: $id, location: $location, start: $start, end: $end, significance: $significance})";
        self.graph_client
            .run(
                query(QUERY)
                    .param("id", event.id)
                    .param("location", String::from(event.location))
                    .param("start", event.duration.0 as i64)
                    .param("end", event.duration.1 as i64)
                    .param("significance", event.significance),
            )
            .await?;
        Ok(())
    }

    async fn link_events(&self, parent_id: i64, child_id: i64) -> Result<(), EventHandlerError> {
        const QUERY: &str = "\
            MATCH (e1:Event {id: $parent_id}), (e2:Event {id: $child_id}) \
            MERGE (e1)-[:DEPENDS_ON]->(e2)";
        self.graph_client
            .run(query(QUERY).param("parent_id", parent_id).param("child_id", child_id))
            .await?;
        Ok(())
    }

    async fn link_event_to_entity(&self, event: &Event, entity: &Entity) -> Result<(), EventHandlerError> {
        const QUERY: &str = "\
            MERGE (ent:Entity {label: $label, text: $text}) \
            MERGE (ev:Event {id: $id}) \
            MERGE (ent)-[:RELATED_TO]->(ev)";
        self.graph_client
            .run(
                query(QUERY)
                    .param("label", &entity.label.to_string())
                    .param("text", &entity.text)
                    .param("id", event.id),
            )
            .await?;
        Ok(())
    }
}

// Now, let's move to the test cases which verify our functionality.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings::spacy_bindings::Entity;
    use crate::bindings::spacy_bindings::EntityLabel::Gpe;
    use tokio::sync::OnceCell;
    use std::env;

    static GRAPH_CLIENT: OnceCell<Arc<Graph>> = OnceCell::const_new();

    async fn setup_graph_client() -> Arc<Graph> {
        GRAPH_CLIENT
            .get_or_init(|| async {
                let uri = env::var("NEO4J_URI").unwrap_or_else(|_| "neo4j://localhost:7687".into());
                let username = env::var("NEO4J_USERNAME").unwrap_or_else(|_| "neo4j".into());
                let password = env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".into());
                Graph::new(&uri, &username, &password)
                    .await
                    .expect("Failed to connect to Neo4j")
            })
            .await
            .clone()
    }

    #[tokio::test]
    async fn test_location_distance() {
        let location1 = Location::from((1.0, 1.0, 1.0));
        let location2 = Location::from((1.0, 4.0, 5.0));
        let distance = location1.distance(&location2);
        assert_eq!(distance, 5.0);
    }

    #[tokio::test]
    async fn test_event_is_schedulable() {
        let event1 = Arc::new(Event::new(
            1,
            "unique_id1".to_string(),
            "user_id1".to_string(),
            Utc::now(),
            "header1".to_string(),
            "name1".to_string(),
            Utc::now(),
            Utc::now(),
            HashMap::new(),
            None,
            Location::from((1.0, 1.0, 1.0)),
            10.0,
            Duration(12, 14),
            vec![],
            12,
            14,
            "resource1".to_string(),
            vec![],
        ));
        
        let event2 = Arc::new(Event::new(
            2,
            "unique_id2".to_string(),
            "user_id2".to_string(),
            Utc::now(),
            "header2".to_string(),
            "name2".to_string(),
            Utc::now(),
            Utc::now(),
            HashMap::new(),
            None,
            Location::from((1.0, 4.0, 5.0)),
            20.0,
            Duration(11, 13),
            vec![],
            11,
            13,
            "resource2".to_string(),
            vec![],
        ));
        
        assert!(!Event::new(
            1,
            "unique_id1".to_string(),
            "user_id1".to_string(),
            Utc::now(),
            "header1".to_string(),
            "name1".to_string(),
            Utc::now(),
            Utc::now(),
            HashMap::new(),
            None,
            Location::from((1.0, 1.0, 1.0)),
            10.0,
            Duration(12, 14),
            vec![event1.clone()],  // Use event1 as a dependency
            12,
            14,
            "resource1".to_string(),
            vec![],
        ).is_schedulable());        
        
    }

    #[tokio::test]
    async fn test_add_new_event() {
        let client = setup_graph_client().await;
        let handler = EventHandler::new(client);
    
        // Create a main event
        let main_event = Event::new(
            1,
            "unique_id".to_string(),
            "user_id".to_string(),
            Utc::now(),
            "header".to_string(),
            "name".to_string(),
            Utc::now(),
            Utc::now(),
            HashMap::new(),
            Some(Entity {
                text: "Spain".to_string(),
                label: Gpe,
            }),
            Location::from((40.4168, -3.7038, 0.0)),
            10.0,
            Duration(1685001600, 1685088000),
            vec![],
            1685001600,
            1685088000,
            "resource".to_string(),
            vec![],
        );
        
    
        // Create dependency events
        let dependency_event1 = Arc::new(Event::new(
            2,
            "unique_id2".to_string(),
            "user_id2".to_string(),
            Utc::now(),
            "header2".to_string(),
            "name2".to_string(),
            Utc::now(),
            Utc::now(),
            HashMap::new(),
            Some(Entity {
                text: "Barcelona".to_string(),
                label: Gpe,
            }),
            Location::from((41.3851, 2.1734, 0.0)),
            8.0,
            Duration(1684915200, 1684957200),
            vec![],
            1684915200,
            1684957200,
            "resource2".to_string(),
            vec![],
        ));
        
        let dependency_event2 = Arc::new(Event::new(
            3,
            "unique_id3".to_string(),
            "user_id3".to_string(),
            Utc::now(),
            "header3".to_string(),
            "name3".to_string(),
            Utc::now(),
            Utc::now(),
            HashMap::new(),
            Some(Entity {
                text: "Valencia".to_string(),
                label: Gpe,
            }),
            Location::from((39.4699, -0.3763, 0.0)),
            6.0,
            Duration(1684872000, 1684915200),
            vec![],
            1684872000,
            1684915200,
            "resource3".to_string(),
            vec![],
        ));
        
    
        // Add dependencies to the main event
        let main_event_with_dependencies = Event::new(
            main_event.id,
            main_event.unique_id.clone(),
            main_event.user_id.clone(),
            main_event.time,
            main_event.header.clone(),
            main_event.name.clone(),
            main_event.start_time,
            main_event.end_time,
            main_event.attributes.clone(),
            main_event.event_type.clone(),
            main_event.location,
            main_event.significance,
            main_event.duration,
            vec![dependency_event1, dependency_event2],
            main_event.start,
            main_event.end,
            main_event.resource.clone(),
            main_event.tags.clone(),
        );
    
        // Add the main event to the graph
        assert!(handler.add_new_event(&main_event_with_dependencies).await.is_ok());
    
        // Verify the added event and its dependencies
        let query = "\
            MATCH (e:Event {id: $id})-[:DEPENDS_ON]->(d:Event)
            RETURN e.id AS event_id, d.id AS dependency_id";
    
        let result = handler
            .graph_client
            .run(query(query).param("id", main_event.id))
            .await
            .unwrap();
    
        let mut event_dependencies = Vec::new();
        for row in result {
            let event_id: i64 = row.get("event_id").unwrap();
            let dependency_id: i64 = row.get("dependency_id").unwrap();
            event_dependencies.push((event_id, dependency_id));
        }
    
        assert_eq!(event_dependencies.len(), 2);
        assert!(event_dependencies.contains(&(main_event.id, dependency_event1.id)));
        assert!(event_dependencies.contains(&(main_event.id, dependency_event2.id)));
    }
}
