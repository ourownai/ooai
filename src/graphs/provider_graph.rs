use std::collections::HashMap;
use std::fmt;

/// Represents a service provider with a name and a set of capabilities.
pub struct Provider {
    pub name: String,
    pub capabilities: HashMap<String, Capability>,
}

impl Provider {
    /// Creates a new provider with the given name and capabilities.
    pub fn new(name: String, capabilities: HashMap<String, Capability>) -> Self {
        Self { name, capabilities }
    }
}

impl fmt::Debug for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Provider {{ name: {}, capabilities: {:?} }}", self.name, self.capabilities)
    }
}

/// Represents a specific functionality offered by a provider,
/// including the endpoints to access it and any required parameters.
pub struct Capability {
    pub name: String,
    pub endpoints: Vec<String>,
    pub parameters: HashMap<String, String>,
}

impl Capability {
    /// Creates a new capability with the given name, endpoints, and parameters.
    pub fn new(name: String, endpoints: Vec<String>, parameters: HashMap<String, String>) -> Self {
        Self { name, endpoints, parameters }
    }
}

impl fmt::Debug for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Capability {{ name: {}, endpoints: {:?}, parameters: {:?} }}", self.name, self.endpoints, self.parameters)
    }
}

/// A registry for managing multiple providers.
pub struct Providers {
    providers: HashMap<String, Provider>,
}

impl Providers {
    /// Constructs a new, empty registry of providers.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Registers a new provider in the registry.
    pub fn register(&mut self, provider: Provider) {
        self.providers.insert(provider.name.clone(), provider);
    }

    /// Retrieves a provider by name, if it exists.
    pub fn get(&self, name: &str) -> Option<&Provider> {
        self.providers.get(name)
    }

    /// Registers a capability under a given provider, returning an error if the provider does not exist.
    pub fn register_capability(&mut self, provider_name: &str, capability: Capability) -> Result<(), String> {
        if let Some(provider) = self.providers.get_mut(provider_name) {
            provider.capabilities.insert(capability.name.clone(), capability);
            Ok(())
        } else {
            Err(format!("Provider '{}' does not exist, capability '{}' not registered.", provider_name, capability.name))
        }
    }

    /// Retrieves a specific capability of a provider by names, if both exist.
    pub fn get_capability(&self, provider_name: &str, capability_name: &str) -> Option<&Capability> {
        self.providers
            .get(provider_name)
            .and_then(|provider| provider.capabilities.get(capability_name))
    }

    /// Removes a provider from the registry, returning the removed provider if it existed.
    pub fn remove_provider(&mut self, provider_name: &str) -> Option<Provider> {
        self.providers.remove(provider_name)
    }

    /// Removes a specific capability from a provider, returning the removed capability if it existed.
    pub fn remove_capability(&mut self, provider_name: &str, capability_name: &str) -> Option<Capability> {
        self.providers
            .get_mut(provider_name)
            .and_then(|provider| provider.capabilities.remove(capability_name))
    }

    /// Lists all capabilities of a given provider, if it exists.
    pub fn list_capabilities(&self, provider_name: &str) -> Option<Vec<&Capability>> {
        self.providers
            .get(provider_name)
            .map(|provider| provider.capabilities.values().collect())
    }

    /// Lists all providers in the registry.
    pub fn list_providers(&self) -> Vec<&Provider> {
        self.providers.values().collect()
    }
}

/// Helper function to create a new provider with no capabilities.
pub fn create_dynamic_provider(name: &str) -> Provider {
    Provider::new(name.to_string(), HashMap::new())
}

/// Helper function to create a new capability with specified endpoints and parameters.
pub fn create_dynamic_capability(
    name: &str,
    endpoints: Vec<String>,
    parameters: HashMap<String, String>,
) -> Capability {
    Capability::new(name.to_string(), endpoints, parameters)
}

fn main() {
    // Create a new Providers registry
    let mut providers = Providers::new();

    // Create a provider for weather information
    let weather_provider = create_dynamic_provider("weather");

    // Register the weather provider
    providers.register(weather_provider);

    // Create capabilities for the weather provider
    let current_weather_capability = create_dynamic_capability(
        "current",
        vec!["https://weather.api/current".to_string()],
        HashMap::from([("units".to_string(), "metric".to_string())]),
    );

    let forecast_capability = create_dynamic_capability(
        "forecast",
        vec!["https://weather.api/forecast".to_string()],
        HashMap::from([("days".to_string(), "7".to_string())]),
    );

    // Register capabilities under the weather provider
    providers.register_capability("weather", current_weather_capability).unwrap();
    providers.register_capability("weather", forecast_capability).unwrap();

    // Retrieve and use a specific capability
    if let Some(capability) = providers.get_capability("weather", "current") {
        println!("Using capability '{}' with endpoints: {:?}", capability.name, capability.endpoints);
        // Here you could imagine making a request to the capability's endpoint
    }

    // List all providers
    let provider_list = providers.list_providers();
    println!("Registered providers: {:?}", provider_list);

    // List all capabilities of the weather provider
    if let Some(capabilities) = providers.list_capabilities("weather") {
        println!("Capabilities of 'weather' provider: {:?}", capabilities);
    }
}