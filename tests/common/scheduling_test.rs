#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_find_events() {
        let mut itinerary = Itinerary::new(vec![]);
        let event = Dependency {
            event: Event { start: 10, end: 12, resource: "Resource1".to_string() },
            dependencies: vec![],
        };
        itinerary.insert_event(event);

        let events = itinerary.find_events_by_resource("Resource1");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].start, 10);
        assert_eq!(events[0].end, 12);
    }

    // Add more tests for other functionalities like find_overlapping_events, calculate_total_duration, etc.
}
