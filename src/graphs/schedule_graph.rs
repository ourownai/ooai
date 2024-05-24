use std::collections::HashMap;
use std::cmp::Ordering;

use crate::event::Event;

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
struct Dependency {
    event: Event,
    dependencies: Vec<Event>,
}

struct Itinerary {
    events: Vec<Dependency>,
    schedules: HashMap<String, Vec<Event>>,
}

impl Itinerary {
    fn new(events: Vec<Dependency>) -> Itinerary {
        let schedules = schedule_resources(events.clone());
        Itinerary { events, schedules }
    }

    fn insert_event(&mut self, event: Dependency) {
        self.events.push(event);
        self.reschedule();
    }

    fn modify_event(&mut self, index: usize, event: Dependency) {
        self.events[index] = event;
        self.reschedule();
    }

    fn remove_event(&mut self, index: usize) {
        self.events.remove(index);
        self.reschedule();
    }

    fn reschedule(&mut self) {
        self.events.sort_by(|a, b| a.event.start.cmp(&b.event.start));
        self.schedules = schedule_resources(self.events.clone());
    }

    fn print_schedules(&self) {
        for (resource, events) in &self.schedules {
            println!("Resource: {}", resource);
            for event in events {
                println!(" {:?}", event);
            }
        }
    }

    fn find_events_by_resource(&self, resource: &str) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|dependency| dependency.event.resource == resource)
            .map(|dependency| &dependency.event)
            .collect()
    }

    fn find_overlapping_events(&self) -> Vec<(&Event, &Event)> {
        let mut overlapping_events = Vec::new();
        for i in 0..self.events.len() {
            for j in i+1..self.events.len() {
                let event1 = &self.events[i].event;
                let event2 = &self.events[j].event;
                if event1.start < event2.end && event2.start < event1.end {
                    overlapping_events.push((event1, event2));
                }
            }
        }
        overlapping_events
    }

    fn calculate_total_duration(&self) -> i32 {
        self.events
            .iter()
            .map(|dependency| dependency.event.end - dependency.event.start)
            .sum()
    }

    fn find_free_time_slots(&self, resource: &str, min_duration: i32) -> Vec<(i32, i32)> {
        let events = self.find_events_by_resource(resource);
        let mut free_time_slots = Vec::new();
        let mut start_time = 0;
        for event in events {
            if event.start - start_time >= min_duration {
                free_time_slots.push((start_time, event.start));
            }
            start_time = event.end;
        }
        free_time_slots
    }

    // Novel method: Find events within a specific time range
    fn find_events_in_range(&self, start: i32, end: i32) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|dependency| dependency.event.start >= start && dependency.event.end <= end)
            .map(|dependency| &dependency.event)
            .collect()
    }

    // Novel method: Find events that depend on a specific event
    fn find_dependent_events(&self, event: &Event) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|dependency| dependency.dependencies.contains(event))
            .map(|dependency| &dependency.event)
            .collect()
    }

    // Novel method: Calculate the critical path (longest path) in the itinerary
    fn calculate_critical_path(&self) -> Vec<&Event> {
        let mut critical_path = Vec::new();
        let mut max_end_time = 0;

        for dependency in &self.events {
            let mut path_end_time = dependency.event.end;
            for dep_event in &dependency.dependencies {
                path_end_time = path_end_time.max(dep_event.end);
            }
            if path_end_time > max_end_time {
                max_end_time = path_end_time;
                critical_path.clear();
                critical_path.push(&dependency.event);
            } else if path_end_time == max_end_time {
                critical_path.push(&dependency.event);
            }
        }

        critical_path
    }
}

fn schedule_resources(events: Vec<Dependency>) -> HashMap<String, Vec<Event>> {
    let mut events = events;
    events.sort_by(|a, b| a.event.start.cmp(&b.event.start));
    let mut schedules = HashMap::new();
    let mut end_time = 0;

    for event in events {
        let mut can_schedule = true;
        for dependency in &event.dependencies {
            if dependency.end > event.event.start {
                can_schedule = false;
                break;
            }
        }
        if can_schedule {
            let resource = event.event.resource.clone();
            schedules
                .entry(resource)
                .or_insert(Vec::new())
                .push(event.event.clone());
            end_time = event.event.end;
        } else {
            let break_event = Event {
                start: end_time,
                end: event.event.start,
                resource: "break".to_string(),
                unique_id: "break".to_string(),
                user_id: "".to_string(),
                time: 0,
                header: "".to_string(),
                event_type: "".to_string(),
                id: "".to_string(),
                name: "break".to_string(),
                location: "break".to_string(),
                start_time: end_time,
                end_time: event.event.start,
                significance: 0,
                attributes: HashMap::new(),
                duration: event.event.start - end_time,
                dependencies: vec![],
                tags: vec![],
            };
            schedules
                .entry(break_event.resource.clone())
                .or_insert(Vec::new())
                .push(break_event);
        }
    }

    schedules
}

pub fn generate_schedules() -> Result<(), Box<dyn std::error::Error>> {
    // Define the events
    let events = vec![
        Dependency {
            event: Event {
                start: 1,
                end: 2,
                resource: "A".to_string(),
                unique_id: "A".to_string(),
                user_id: "".to_string(),
                time: 0,
                header: "".to_string(),
                event_type: "".to_string(),
                id: "".to_string(),
                name: "A".to_string(),
                location: "A".to_string(),
                start_time: 1,
                end_time: 2,
                significance: 0,
                attributes: HashMap::new(),
                duration: 1,
                dependencies: vec![],
                tags: vec![],
            },
            dependencies: vec![],
        },
        Dependency {
            event: Event {
                start: 2,
                end: 4,
                resource: "B".to_string(),
                unique_id: "B".to_string(),
                user_id: "".to_string(),
                time: 0,
                header: "".to_string(),
                event_type: "".to_string(),
                id: "".to_string(),
                name: "B".to_string(),
                location: "B".to_string(),
                start_time: 2,
                end_time: 4,
                significance: 0,
                attributes: HashMap::new(),
                duration: 2,
                dependencies: vec![],
                tags: vec![],
            },
            dependencies: vec![],
        },
        Dependency {
            event: Event {
                start: 4,
                end: 6,
                resource: "C".to_string(),
                unique_id: "C".to_string(),
                user_id: "".to_string(),
                time: 0,
                header: "".to_string(),
                event_type: "".to_string(),
                id: "".to_string(),
                name: "C".to_string(),
                location: "C".to_string(),
                start_time: 4,
                end_time: 6,
                significance: 0,
                attributes: HashMap::new(),
                duration: 2,
                dependencies: vec![],
                tags: vec![],
            },
            dependencies: vec![],
        },
        Dependency {
            event: Event {
                start: 3,
                end: 5,
                resource: "D".to_string(),
                unique_id: "D".to_string(),
                user_id: "".to_string(),
                time: 0,
                header: "".to_string(),
                event_type: "".to_string(),
                id: "".to_string(),
                name: "D".to_string(),
                location: "D".to_string(),
                start_time: 3,
                end_time: 5,
                significance: 0,
                attributes: HashMap::new(),
                duration: 2,
                dependencies: vec![],
                tags: vec![],
            },
            dependencies: vec![],
        },
    ];

    // Create an itinerary from the events
    let mut itinerary = Itinerary::new(events);

    // Print the initial schedules
    println!("Initial schedules:");
    itinerary.print_schedules();

    // Insert a new event into the itinerary
    let new_event = Dependency {
        event: Event {
            start: 7,
            end: 8,
            resource: "E".to_string(),
            unique_id: "E".to_string(),
            user_id: "".to_string(),
            time: 0,
            header: "".to_string(),
            event_type: "".to_string(),
            id: "".to_string(),
            name: "E".to_string(),
            location: "E".to_string(),
            start_time: 7,
            end_time: 8,
            significance: 0,
            attributes: HashMap::new(),
            duration: 1,
            dependencies: vec![],
            tags: vec![],
        },
        dependencies: vec![],
    };
    itinerary.insert_event(new_event);

    // Print the updated schedules after inserting a new event
    println!("\nUpdated schedules after inserting a new event:");
    itinerary.print_schedules();

    // Modify an existing event in the itinerary
    let modified_event = Dependency {
        event: Event {
            start: 2,
            end: 3,
            resource: "B".to_string(),
            unique_id: "B".to_string(),
            user_id: "".to_string(),
            time: 0,
            header: "".to_string(),
            event_type: "".to_string(),
            id: "".to_string(),
            name: "B".to_string(),
            location: "B".to_string(),
            start_time: 2,
            end_time: 3,
            significance: 0,
            attributes: HashMap::new(),
            duration: 1,
            dependencies: vec![],
            tags: vec![],
        },
        dependencies: vec![],
    };
    itinerary.modify_event(1, modified_event);

    // Print the updated schedules after modifying an event
    println!("\nUpdated schedules after modifying an event:");
    itinerary.print_schedules();

    // Remove an event from the itinerary
    itinerary.remove_event(2);

    // Print the updated schedules after removing an event
    println!("\nUpdated schedules after removing an event:");
    itinerary.print_schedules();

    // Find events by resource
    let resource = "B";
    let events_by_resource = itinerary.find_events_by_resource(resource);
    println!("\nEvents for resource '{}':", resource);
    for event in events_by_resource {
        println!(" {:?}", event);
    }

    // Find overlapping events
    let overlapping_events = itinerary.find_overlapping_events();
    println!("\nOverlapping events:");
    for (event1, event2) in overlapping_events {
        println!(" {:?} overlaps with {:?}", event1, event2);
    }

    // Calculate total duration of events
    let total_duration = itinerary.calculate_total_duration();
    println!("\nTotal duration of events: {}", total_duration);

    // Find free time slots for a resource
    let resource = "A";
    let min_duration = 2;
    let free_time_slots = itinerary.find_free_time_slots(resource, min_duration);
    println!("\nFree time slots for resource '{}' with minimum duration {}:", resource, min_duration);
    for (start, end) in free_time_slots {
        println!(" Start: {}, End: {}", start, end);
    }

    // Find events within a specific time range
    let start_time = 2;
    let end_time = 5;
    let events_in_range = itinerary.find_events_in_range(start_time, end_time);
    println!("\nEvents within time range {} to {}:", start_time, end_time);
    for event in events_in_range {
        println!(" {:?}", event);
    }

    // Find events that depend on a specific event
    let event = &itinerary.events[0].event;
    let dependent_events = itinerary.find_dependent_events(event);
    println!("\nEvents dependent on {:?}:", event);
    for event in dependent_events {
        println!(" {:?}", event);
    }

    // Calculate the critical path in the itinerary
    let critical_path = itinerary.calculate_critical_path();
    println!("\nCritical path:");
    for event in critical_path {
        println!(" {:?}", event);
    }

    Ok(())
}
