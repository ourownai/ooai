use std::collections::HashMap;
use std::time::SystemTime;

pub struct UserData {
    pub preferences: HashMap<String, String>,
    pub profile: HashMap<String, String>,
    pub history: Vec<(SystemTime, String)>,
}

impl UserData {
    pub fn new() -> Self {
        Self {
            preferences: HashMap::new(),
            profile: HashMap::new(),
            history: Vec::new(),
        }
    }

    pub fn set_preference(&mut self, key: String, value: String) {
        self.preferences.insert(key, value);
    }

    pub fn set_profile(&mut self, key: String, value: String) {
        self.profile.insert(key, value);
    }

    pub fn add_history(&mut self, timestamp: SystemTime, event: String) {
        self.history.push((timestamp, event));
    }
}