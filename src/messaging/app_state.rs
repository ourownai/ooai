use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct AppState {
    routing_table: Arc<Mutex<HashMap<String, String>>>,
    // Add other necessary fields
}

impl AppState {
    pub fn new() -> Self {
        Self {
            routing_table: Arc::new(Mutex::new(HashMap::new())),
            // Initialize other fields
        }
    }

    pub async fn get_routing_table(&self) -> HashMap<String, String> {
        let routing_table = self.routing_table.lock().unwrap();
        routing_table.clone()
    }

    // Implement other methods as needed
}
