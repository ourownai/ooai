use rasa_rust_sdk::{FromAction, FromSender, Sender};

pub struct TedPolicyConfig {
    num_layers: Option<usize>,
    hidden_size: Option<usize>,
    learning_rate: Option<f32>,
}

impl TedPolicyConfig {
    pub fn new() -> TedPolicyConfig {
        TedPolicyConfig {
            num_layers: None,
            hidden_size: None,
            learning_rate: None,
        }
    }

    pub fn set_num_layers(&mut self, num_layers: usize) {
        self.num_layers = Some(num_layers);
    }

    pub fn set_hidden_size(&mut self, hidden_size: usize) {
        self.hidden_size = Some(hidden_size);
    }

    pub fn set_learning_rate(&mut self, learning_rate: f32) {
        self.learning_rate = Some(learning_rate);
    }

    pub fn build(&self, sender: &Sender) {
        let mut config = String::new();

        if let Some(num_layers) = self.num_layers {
            config.push_str(&format!("num_layers: {}\n", num_layers));
        }

        if let Some(hidden_size) = self.hidden_size {
            config.push_str(&format!("hidden_size: {}\n", hidden_size));
        }

        if let Some(learning_rate) = self.learning_rate {
            config.push_str(&format!("learning_rate: {}\n", learning_rate));
        }

        let message = format!(
            "Here is the current TED policy configuration:\n{}",
            config
        );

        sender.send_message(message);
    }
}

impl FromAction for TedPolicyConfig {
    fn from_action(sender: &Sender, action: &str) -> Option<TedPolicyConfig> {
        match action {
            "set_num_layers" => Some(TedPolicyConfig::new()),
            "set_hidden_size" => Some(TedPolicyConfig::new()),
            "set_learning_rate" => Some(TedPolicyConfig::new()),
            "build" => Some(TedPolicyConfig::new()),
            _ => None,
        }
    }
}

impl FromSender for TedPolicyConfig {
    fn from_sender(sender: &Sender) -> Option<TedPolicyConfig> {
        Some(TedPolicyConfig::new())
    }
}


/*
The TedPolicyConfig struct represents the configuration for the TED policy.
It has three optional fields for the number of layers, hidden size, and learning rate.
The new() function creates a new instance of the TedPolicyConfig struct with all fields set to None.
The set_num_layers(), set_hidden_size(), and set_learning_rate() functions allow the user to set the corresponding fields.
The build() function generates a string representation
*/
