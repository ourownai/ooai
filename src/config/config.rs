use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use semver::Version;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    version: String,
    user_types: HashMap<String, UserType>,
    preference_types: HashMap<String, PreferenceType>,
    modules: HashMap<String, ModuleConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserType {
    warning_levels: HashMap<String, u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PreferenceType {
    description: String,
    value_type: String,
    default_value: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModuleConfig {
    enabled: bool,
    settings: HashMap<String, Value>,
    overrides: HashMap<String, HashMap<String, Value>>,
}

impl Config {
    fn load(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        let version = Version::parse(&self.version).map_err(|e| e.to_string())?;
        // Perform additional validation checks on the configuration data
        // Return an error if any validation fails
        Ok(())
    }

    fn get_user_type(&self, user_type: &str) -> Option<&UserType> {
        self.user_types.get(user_type)
    }

    fn get_preference_type(&self, preference_type: &str) -> Option<&PreferenceType> {
        self.preference_types.get(preference_type)
    }

    fn get_module_config(&self, module_name: &str) -> Option<&ModuleConfig> {
        self.modules.get(module_name)
    }

    fn is_module_enabled(&self, module_name: &str) -> bool {
        self.get_module_config(module_name)
            .map(|config| config.enabled)
            .unwrap_or(false)
    }

    fn get_module_setting(&self, module_name: &str, key: &str, user_type: &str) -> Option<&Value> {
        let module_config = self.get_module_config(module_name)?;
        module_config.overrides.get(user_type)?.get(key)
            .or_else(|| module_config.settings.get(key))
    }

    fn check_warning_level(&self, module_name: &str, key: &str, user_type: &str) -> Option<u8> {
        let user_type_config = self.get_user_type(user_type)?;
        user_type_config.warning_levels.get(key).copied()
    }

    fn get_all_enabled_modules(&self) -> Vec<&str> {
        self.modules
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(name, _)| name.as_str())
            .collect()
    }

    fn get_all_user_types(&self) -> Vec<&str> {
        self.user_types.keys().map(|s| s.as_str()).collect()
    }

    fn get_all_preference_types(&self) -> Vec<&str> {
        self.preference_types.keys().map(|s| s.as_str()).collect()
    }
}

fn main() {
    let config_file = "config.json";
    let config = Config::load(config_file).unwrap();
    match config.validate() {
        Ok(_) => println!("Configuration is valid."),
        Err(e) => {
            eprintln!("Configuration validation failed: {}", e);
            std::process::exit(1);
        }
    }

    let user_type = "normal";

    // Access module configurations dynamically
    for module_name in config.get_all_enabled_modules() {
        println!("{} module is enabled.", module_name);
        if let Some(value) = config.get_module_setting(module_name, "default_provider", user_type) {
            println!("Default provider for {}: {}", module_name, value);
        }
        if let Some(warning_level) = config.check_warning_level(module_name, "default_provider", user_type) {
            println!("Warning level for {} default provider: {}", module_name, warning_level);
        }
    }

    // Access user types dynamically
    println!("Available user types:");
    for user_type in config.get_all_user_types() {
        println!("- {}", user_type);
    }

    // Access preference types dynamically
    println!("Available preference types:");
    for preference_type in config.get_all_preference_types() {
        if let Some(pref_type) = config.get_preference_type(preference_type) {
            println!("- {}: {}", preference_type, pref_type.description);
        }
    }

    // Use the configuration values in your script logic
    // ...
}