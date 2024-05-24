//! # Event and Alert Management Module
//!
//! This module defines structs and methods for managing events and alerts.
//!
//! ## Structs
//!
//! - [`Event`]: Represents an event with a location, significance, and tags.
//!   - `location`: A tuple of three floats representing the event's location.
//!   - `significance`: A float representing the event's significance.
//!   - `tags`: A vector of strings representing the tags associated with the event.
//!
//! - [`Alert`]: Represents an alert message for an event.
//!   - `event_name`: A string representing the name of the event.
//!   - `message`: A string representing the alert message.
//!   - `severity`: A float representing the severity of the alert.
//!
//! - [`EventGraph`]: Represents a graph of events and alerts.
//!   - `events`: A `HashMap` of events keyed by their name.
//!   - `alerts`: A `Vec` of alerts for events.
//!
//! ## Methods
//!
//! - [`EventGraph::new()`]: Creates a new instance of the `EventGraph` struct with an empty event list and alert list.
//!
//! - [`EventGraph::add_event()`]: Adds a new event to the graph with the given name, location, significance, and tags.
//!
//! - [`EventGraph::get_weighted_graph()`]: Calculates the weight of each event based on the agent's preferences, which are provided as a `HashMap` of event names to preference values.
//!
//! - [`EventGraph::get_nearby_events()`]: Finds all events within a certain distance of a given location.
//!
//! - [`EventGraph::add_alerts_along_path()`]: Adds alerts to the `alerts` vector in `self` if there are any events along a given path.
//!
//! - [`EventGraph::generate_alert()`]: Generates an alert for a given event if it is in the weighted graph.
//!
//! - [`EventGraph::calculate_significance_stats()`]: Calculates the mean and standard deviation of event significance.
//!
//! - [`EventGraph::calculate_distance()`]: Calculates the distance between two locations.
//!
//! - [`EventGraph::calculate_alert_severity()`]: Calculates the severity of an alert based on the weight of the event and the maximum weight of all events.
//!
//! - [`EventGraph::filter_events_by_tags()`]: Filters events based on a given set of tags and returns a vector of events that have at least one of the specified tags.
//!
//! - [`EventGraph::get_events_sorted_by_significance()`]: Returns a vector of events sorted by their significance in descending order.
//!
//! - [`EventGraph::get_top_events_by_weight()`]: Returns the top N events based on their weights calculated using the provided preferences.

use crate::event::Event;

use std::collections::HashMap;

pub struct Alert {
    pub event_name: String,
    pub message: String,
    pub severity: f32,
}

pub struct EventGraph {
    pub events: HashMap<String, Event>,
    pub alerts: Vec<Alert>,
}

impl EventGraph {
    pub fn new() -> Self {
        EventGraph {
            events: HashMap::new(),
            alerts: Vec::new(),
        }
    }

    pub fn add_event(
        &mut self,
        unique_id: String,
        user_id: String,
        time: u64,
        header: String,
        event_type: String,
        id: String,
        name: String,
        start_time: u64,
        end_time: u64,
        attributes: HashMap<String, String>,
        duration: u64,
        dependencies: Vec<String>,
        start: u64,
        end: u64,
        resource: String,
        location: (f32, f32, f32),
        significance: f32,
        tags: Vec<String>,
    ) {
        let event = Event {
            unique_id,
            user_id,
            time,
            header,
            event_type,
            id,
            name,
            start_time,
            end_time,
            attributes,
            duration,
            dependencies,
            start,
            end,
            resource,
            location,
            significance,
            tags,
        };
        self.events.insert(name, event);
    }

    pub fn get_weighted_graph(&self, preferences: &HashMap<String, f32>) -> HashMap<String, f32> {
        let mut weighted_graph = HashMap::new();
        let (mean, std_dev) = self.calculate_significance_stats();

        for (name, event) in self.events.iter() {
            let deviation = (event.significance - mean) / std_dev;
            let preference = preferences.get(name).unwrap_or(&0.0);
            let weight = event.significance * (1.0 + deviation) * preference;
            weighted_graph.insert(name.to_string(), weight);
        }

        weighted_graph
    }

    pub fn get_nearby_events(&self, location: (f32, f32, f32), max_distance: f32) -> Vec<String> {
        let mut nearby_events = Vec::new();

        for (name, event) in self.events.iter() {
            let distance = self.calculate_distance(event.location, location);
            if distance <= max_distance {
                nearby_events.push(name.to_string());
            }
        }

        nearby_events
    }

    pub fn add_alerts_along_path(
        &mut self,
        path: &[(f32, f32, f32)],
        max_distance: f32,
        preferences: &HashMap<String, f32>,
    ) {
        let weighted_graph = self.get_weighted_graph(preferences);

        for location in path {
            let nearby_events = self.get_nearby_events(*location, max_distance);

            for event_name in nearby_events {
                if let Some(alert) = self.generate_alert(&event_name, &weighted_graph) {
                    self.alerts.push(alert);
                }
            }
        }
    }

    pub fn generate_alert(
        &self,
        event_name: &str,
        weighted_graph: &HashMap<String, f32>,
    ) -> Option<Alert> {
        if let Some(event) = self.events.get(event_name) {
            if let Some(weight) = weighted_graph.get(event_name) {
                let severity = self.calculate_alert_severity(event, *weight);
                let message = format!(
                    "Alert: Event {} is nearby and has weight {}.",
                    event_name, weight
                );
                return Some(Alert {
                    event_name: event_name.to_string(),
                    message,
                    severity,
                });
            }
        }

        None
    }

    fn calculate_significance_stats(&self) -> (f32, f32) {
        let mut sum = 0.0;
        let mut count = 0;

        for (_, event) in self.events.iter() {
            sum += event.significance;
            count += 1;
        }

        let mean = sum / count as f32;
        let mut variance = 0.0;

        for (_, event) in self.events.iter() {
            variance += (event.significance - mean).powi(2);
        }

        let std_dev = (variance / count as f32).sqrt();

        (mean, std_dev)
    }

    fn calculate_distance(&self, location1: (f32, f32, f32), location2: (f32, f32, f32)) -> f32 {
        ((location1.0 - location2.0).powi(2)
            + (location1.1 - location2.1).powi(2)
            + (location1.2 - location2.2).powi(2))
            .sqrt()
    }

    fn calculate_alert_severity(&self, event: &Event, weight: f32) -> f32 {
        let max_weight = self.events.values().map(|e| e.significance).fold(0.0, f32::max);
        weight / max_weight
    }

    pub fn filter_events_by_tags(&self, tags: &[String]) -> Vec<&Event> {
        self.events
            .values()
            .filter(|event| event.tags.iter().any(|tag| tags.contains(tag)))
            .collect()
    }

    pub fn get_events_sorted_by_significance(&self) -> Vec<&Event> {
        let mut events: Vec<&Event> = self.events.values().collect();
        events.sort_by(|a, b| b.significance.partial_cmp(&a.significance).unwrap());
        events
    }

    pub fn get_top_events_by_weight(&self, preferences: &HashMap<String, f32>, n: usize) -> Vec<(&str, f32)> {
        let weighted_graph = self.get_weighted_graph(preferences);
        let mut events: Vec<(&str, f32)> = weighted_graph.iter().map(|(name, weight)| (name.as_str(), *weight)).collect();
        events.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        events.into_iter().take(n).collect()
    }
}
