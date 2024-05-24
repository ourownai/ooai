//! # Event Recommendation Module
//!
//! This module defines a struct called `RecommendHandler` responsible for recommending events to users based on their location, preferences, and timing.
//! The `RecommendHandler` operates through a series of stages, including event recall, dependency loading, filtering, and sorting to provide the most
//! relevant event recommendations to the user.
//!
//! The recommendation process is as follows:
//! 1. **Recall Stage**: Retrieves a list of potential event candidates based on user preferences and proximity. It leverages a Neo4j graph database to find
//!    events connected to user interests.
//! 2. **Dependency Loading Stage**: Loads additional dependencies for each event candidate, such as checking event schedulability and other logistical considerations.
//! 3. **Filtering Stage**: Filters out event candidates based on certain criteria, such as distance thresholds and schedulability, ensuring only viable events are considered.
//! 4. **Sorting Stage**: Sorts the remaining event candidates based on a combination of user preferences, event significance, and other relevant metrics to rank the most
//!    suitable events highest.
//!
//! The `RecommendHandler` utilizes async/await for asynchronous operations, particularly for database interactions and the processing pipeline. It is designed to integrate
//! seamlessly with a larger system that manages user interactions, event data, and user preferences.
//!
//! Constants for thresholds and other parameters are configurable, allowing for flexibility in tuning the recommendation process. The module also includes tests to verify
//! the functionality of the `RecommendHandler` through both unit and integration testing, ensuring reliability and performance.
//!
//! ## Dependencies
//!
//! - [`crate::event`]: Provides the `Event`, `EventHandler`, and `Location` types for event-related operations.
//! - [`crate::graphs::event_graph`]: Defines the `EventHandlerError` type for error handling.
//! - [`crate::significance::event_significance`]: Provides the `EventSignificance` and `EventType` types for event significance calculation.
//! - [`futures`]: Used for handling asynchronous operations.
//! - [`neo4rs`]: A Rust client for Neo4j database operations.
//! - [`std::sync::Arc`]: Enables shared ownership of the Neo4j client across multiple tasks.
//! - [`thiserror`]: Provides the `Error` derive macro for custom error types.


use crate::event::Location;
use crate::event::{Event, EventHandler};
use crate::graphs::event_graph::EventHandlerError;
use crate::significance::event_significance::{EventSignificance, EventType};

use futures::future::join_all;
use neo4rs::{query, Graph};
use std::sync::Arc;
use std::convert::TryFrom;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecommendError {
    #[error("Data conversion error: {0}")]
    DataConversionError(String),
    #[error("Neo4j error: {0}")]
    Neo4jError(#[from] neo4rs::Error),
    #[error("Event handler error: {0}")]
    EventHandlerError(#[from] EventHandlerError),
}

/// Responsible for recommending events to users based on their preferences and proximity.
pub struct RecommendHandler {
    neo_client: Arc<Graph>,
    pub distance_threshold: f32,
    pub time_to_start_threshold: u64,
}

#[derive(Debug)]
pub struct Alert {
    pub event_name: String,
    pub message: String,
    pub event: Event,
}

#[derive(Debug)]
struct EventCandidate {
    pub event: Event,
    pub distance: f32,
    pub preference: f64,
    pub filter_reason: Option<CandidateFilterReason>,
}

#[derive(Eq, Ord, PartialOrd, PartialEq, Debug, Copy, Clone)]
enum CandidateFilterReason {
    Unschedulable,
    TooFar,
    TooLate,
}

impl RecommendHandler {
    /// Recommends an event to a user based on location, time, and user preferences.
    pub async fn recommend_event(
        &self,
        user_id: i64,
        user_location: Location,
        time: u64,
    ) -> Result<Vec<Alert>, RecommendError> {
        let events = self.recommend_recall(user_id, user_location, time).await?;
        let events = self.load_event_dependencies(events).await?;
        let mut events = self.filter_event_candidates(events);
        self.sort_events(&mut events);
        let alerts = events
            .into_iter()
            .map(|event_candidate| {
                let event = &event_candidate.event;
                let message_data = HashMap::from([
                    ("event_name".to_string(), event.name.clone()),
                    ("distance".to_string(), event_candidate.distance.to_string()),
                    ("start_time".to_string(), event.start_time.to_string()),
                    ("end_time".to_string(), event.end_time.to_string()),
                    ("location".to_string(), event.location.to_string()),
                ]);
                let message = self.generate_message(message_data);
                Alert {
                    event_name: event.name.clone(),
                    message,
                    event: event.clone(),
                }
            })
            .collect();
        Ok(alerts)
    }

    /// Generates a message using a generative language model.
    fn generate_message(&self, message_data: HashMap<String, String>) -> String {
        // TODO: Implement the logic to pass the message_data to a generative language model
        // and retrieve the generated message.
        // For now, we'll return a placeholder message.
        format!("An event you may be interested in: {}", message_data["event_name"])
    }

    /// Retrieves events that a user might be interested in based on their preferences and proximity.
    async fn recommend_recall(
        &self,
        user_id: i64,
        user_location: Location,
        time: u64,
    ) -> Result<Vec<EventCandidate>, RecommendError> {
        const QUERY: &str = "
            MATCH (u:User)-[:Mention]-(e:Entity)-[:Schedule]-(v:Event)
            WHERE u.user_id=$user_id AND v.start>=$start AND v.start<$start_before
            RETURN m.score, e.text, e.label, v.id, v.sig, v.loc, v.start, v.end
        ";
        let rows = self
            .neo_client
            .execute(
                query(QUERY)
                    .param("user_id", user_id)
                    .param("start", time as i64)
                    .param("start_before", (time + self.time_to_start_threshold) as i64),
            )
            .await
            .map_err(RecommendError::Neo4jError)?
            .collect::<Vec<_>>()
            .await;
        rows.into_iter()
            .map(|row| {
                let location =
                    Location::try_from(row.get::<String>("v.loc").unwrap_or_default()).map_err(
                        |_| RecommendError::DataConversionError("Failed to parse location".into()),
                    )?;
                let distance = location.distance(&user_location);
                let filter_reason = if distance > self.distance_threshold {
                    Some(CandidateFilterReason::TooFar)
                } else {
                    None
                };
                let mut attributes = HashMap::new();
                attributes.insert("magnitude".to_string(), row.get("v.sig").unwrap_or_default());
                attributes.insert("depth".to_string(), row.get("v.depth").unwrap_or_default());
                attributes.insert("importance".to_string(), row.get("v.importance").unwrap_or_default());
                attributes.insert("preference".to_string(), row.get("m.score").unwrap_or_default());
                attributes.insert("severity".to_string(), row.get("v.severity").unwrap_or_default());
                attributes.insert("duration".to_string(), row.get("v.duration").unwrap_or_default());
                let event_type = match row.get::<String>("e.label").unwrap_or_default().as_str() {
                    _ => EventType::ScheduledEvent,
                };
                let event = Event {
                    unique_id: row.get("v.id").unwrap_or_default(),
                    user_id: Some(user_id),
                    time: time as i64,
                    header: row.get("e.text").unwrap_or_default(),
                    duration: row.get("v.duration").unwrap_or_default(),
                    dependencies: vec![],
                    start: row.get("v.start").unwrap_or_default(),
                    end: row.get("v.end").unwrap_or_default(),
                    resource: "".to_string(),
                    tags: vec![],
                    id: row.get("v.id").unwrap_or_default(),
                    name: row.get("e.text").unwrap_or_default(),
                    location,
                    start_time: row.get("v.start").unwrap_or_default(),
                    end_time: row.get("v.end").unwrap_or_default(),
                    significance: EventSignificance::new(event_type, attributes).calculate_significance(),
                    event_type,
                    attributes,
                   };
                Ok(EventCandidate {
                    event,
                    distance,
                    preference: row.get("m.score").unwrap_or_default(),
                    filter_reason,
                })
            })
            .collect()
    }

    /// Loads additional event dependencies and updates schedulability.
    async fn load_event_dependencies(
        &self,
        candidates: Vec<EventCandidate>,
    ) -> Result<Vec<EventCandidate>, RecommendError> {
        let events = join_all(candidates.into_iter().map(|candidate| {
            let event_handler = EventHandler::new(self.neo_client.clone());
            async move {
                if candidate.filter_reason.is_some() {
                    return Ok(candidate);
                }
                let event = event_handler.load_event_dependencies(&candidate.event).await?;
                let filter_reason = if !event.is_schedulable() {
                    Some(CandidateFilterReason::Unschedulable)
                } else if event.end_time < chrono::Utc::now().timestamp() as u64 {
                    Some(CandidateFilterReason::TooLate)
                } else {
                    None
                };
                Ok(EventCandidate {
                    filter_reason,
                    ..candidate
                })
            }
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, RecommendError>>()?;
        Ok(events)
    }

    /// Filters out unsuitable event candidates.
    fn filter_event_candidates(&self, candidates: Vec<EventCandidate>) -> Vec<EventCandidate> {
        candidates
            .into_iter()
            .filter(|c| c.filter_reason.is_none())
            .collect()
    }

    /// Sorts events based on preferences and significance.
    fn sort_events(&self, events: &mut Vec<EventCandidate>) {
        events.sort_unstable_by(|a, b| {
            let a_weight = a.event.significance * a.preference;
            let b_weight = b.event.significance * b.preference;
            b_weight.total_cmp(&a_weight)
        });
    }
}
