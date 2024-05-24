use std::collections::HashMap;

// Define an enum to represent the different types of events that can occur.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    SeismicAnomaly,   // A seismic event
    ScheduledEvent,   // A scheduled event
    AgentPreference,  // A preference expressed by an agent
    AlertGraph,       // An alert on a graph
    CustomEvent(String), // A custom event type
}

// Define a struct to represent the significance of an event, which is determined by its attributes and type.
#[derive(Debug, Clone)]
pub struct EventSignificance {
    attributes: HashMap<String, f64>, // The attributes associated with the event and their values
    event_type: EventType,            // The type of the event
}

impl EventSignificance {
    // Define a constructor for the struct that takes the event type and its attributes.
    pub fn new(event_type: EventType, attributes: HashMap<String, f64>) -> Self {
        Self {
            event_type,
            attributes,
        }
    }

    // Define a method to get the event type.
    pub fn event_type(&self) -> &EventType {
        &self.event_type
    }

    // Define a method to get the value of a particular attribute associated with the event.
    pub fn attribute(&self, attribute_name: &str) -> Option<f64> {
        self.attributes.get(attribute_name).copied()
    }

    // Define a method to calculate the significance of the event based on its type and attributes.
    pub fn calculate_significance(&self) -> f64 {
        match self.event_type {
            // For a seismic event, the significance is based on the magnitude and depth of the event.
            EventType::SeismicAnomaly => {
                let magnitude = self.attribute("magnitude").unwrap_or(0.0);
                let depth = self.attribute("depth").unwrap_or(0.0);
                let location_factor = self.attribute("location_factor").unwrap_or(1.0);
                (magnitude * depth * location_factor) / 100.0
            }
            // For a scheduled event, the significance is based on its importance and urgency.
            EventType::ScheduledEvent => {
                let importance = self.attribute("importance").unwrap_or(0.0);
                let urgency = self.attribute("urgency").unwrap_or(1.0);
                importance * urgency * 10.0
            }
            // For an event representing an agent's preference, the significance is based on the preference expressed and the agent's influence.
            EventType::AgentPreference => {
                let preference = self.attribute("preference").unwrap_or(0.0);
                let agent_influence = self.attribute("agent_influence").unwrap_or(1.0);
                preference * agent_influence * 5.0
            }
            // For an alert on a graph, the significance is based on the severity of the alert, its duration, and the graph's importance.
            EventType::AlertGraph => {
                let severity = self.attribute("severity").unwrap_or(0.0);
                let duration = self.attribute("duration").unwrap_or(0.0);
                let graph_importance = self.attribute("graph_importance").unwrap_or(1.0);
                (severity * duration * graph_importance) / 100.0
            }
            // For a custom event type, the significance is based on a custom calculation.
            EventType::CustomEvent(_) => {
                let custom_factor = self.attribute("custom_factor").unwrap_or(1.0);
                custom_factor * 10.0
            }
        }
    }

    // Define a method to update the value of an attribute.
    pub fn update_attribute(&mut self, attribute_name: &str, value: f64) {
        self.attributes.insert(attribute_name.to_string(), value);
    }

    // Define a method to remove an attribute.
    pub fn remove_attribute(&mut self, attribute_name: &str) {
        self.attributes.remove(attribute_name);
    }

    // Define a method to check if an attribute exists.
    pub fn has_attribute(&self, attribute_name: &str) -> bool {
        self.attributes.contains_key(attribute_name)
    }
}

// Define a struct to represent a collection of events and their significances.
pub struct EventCollection {
    events: Vec<EventSignificance>,
}

impl EventCollection {
    // Define a constructor for the struct that takes a vector of events.
    pub fn new(events: Vec<EventSignificance>) -> Self {
        Self { events }
    }

    // Define a method to add an event to the collection.
    pub fn add_event(&mut self, event: EventSignificance) {
        self.events.push(event);
    }

    // Define a method to remove an event from the collection by index.
    pub fn remove_event(&mut self, index: usize) {
        if index < self.events.len() {
            self.events.remove(index);
        }
    }

    // Define a method to get the total significance of all events in the collection.
    pub fn total_significance(&self) -> f64 {
        self.events.iter().map(|event| event.calculate_significance()).sum()
    }

    // Define a method to get the events of a particular type.
    pub fn events_by_type(&self, event_type: &EventType) -> Vec<&EventSignificance> {
        self.events
            .iter()
            .filter(|event| event.event_type() == event_type)
            .collect()
    }

    // Define a method to get the event with the highest significance.
    pub fn most_significant_event(&self) -> Option<&EventSignificance> {
        self.events
            .iter()
            .max_by(|a, b| a.calculate_significance().partial_cmp(&b.calculate_significance()).unwrap())
    }
}