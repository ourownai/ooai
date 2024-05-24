use std::collections::HashMap;
use crate::provider_types::search::SearchProvider;
use crate::data_exchange::exchange_interfaces::DataExchange;

pub struct WikipediaSearchProvider {
    api_url: String,
    knowledge_graph: HashMap<String, HashMap<String, f32>>,
}

impl WikipediaSearchProvider {
    pub fn new(api_url: String) -> Self {
        let mut knowledge_graph = HashMap::new();
        // Initialize the knowledge graph with predefined weights
        knowledge_graph.insert("programming".to_string(), HashMap::from([
            ("rust".to_string(), 0.8),
            ("python".to_string(), 0.7),
            ("java".to_string(), 0.6),
        ]));
        knowledge_graph.insert("science".to_string(), HashMap::from([
            ("physics".to_string(), 0.9),
            ("chemistry".to_string(), 0.8),
            ("biology".to_string(), 0.7),
        ]));
        // Add more categories and topics to the knowledge graph

        Self {
            api_url,
            knowledge_graph,
        }
    }

    fn update_knowledge_graph(&mut self, category: &str, topic: &str, weight: f32) {
        if let Some(topics) = self.knowledge_graph.get_mut(category) {
            topics.insert(topic.to_string(), weight);
        } else {
            let mut topics = HashMap::new();
            topics.insert(topic.to_string(), weight);
            self.knowledge_graph.insert(category.to_string(), topics);
        }
    }

    fn get_topic_weight(&self, category: &str, topic: &str) -> f32 {
        self.knowledge_graph.get(category).and_then(|topics| topics.get(topic)).cloned().unwrap_or(0.0)
    }

    fn calculate_query_relevance(&self, query: &str) -> f32 {
        let mut relevance = 0.0;
        for (category, topics) in &self.knowledge_graph {
            for (topic, weight) in topics {
                if query.contains(topic) {
                    relevance += weight;
                }
            }
        }
        relevance
    }
}

impl SearchProvider for WikipediaSearchProvider {
    fn search(&self, query: &str) -> Result<HashMap<String, String>, String> {
        let relevance = self.calculate_query_relevance(query);

        let response = reqwest::blocking::get(&self.api_url)
            .map_err(|e| format!("Request failed: {}", e))?
            .query(&[("action", "opensearch"), ("format", "json"), ("limit", "2"), ("search", query)])
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let json: Vec<Vec<String>> = response.json().map_err(|e| format!("Failed to parse JSON: {}", e))?;

        if json.len() >= 4 {
            let urls = &json[3];
            let mut results = HashMap::new();
            for (i, url) in urls.iter().enumerate() {
                results.insert(format!("result_{}", i + 1), url.to_string());
            }

            // Update the knowledge graph based on the search query and relevance
            for (category, topics) in &mut self.knowledge_graph {
                for (topic, weight) in topics {
                    if query.contains(topic) {
                        *weight += relevance;
                    }
                }
            }

            Ok(results)
        } else {
            Err("Unexpected JSON format".to_string())
        }
    }
}

pub struct WikipediaDataExchange {
    search_provider: WikipediaSearchProvider,
}

impl WikipediaDataExchange {
    pub fn new(api_url: String) -> Self {
        Self {
            search_provider: WikipediaSearchProvider::new(api_url),
        }
    }
}

#[async_trait::async_trait]
impl DataExchange<String, Result<HashMap<String, String>, String>> for WikipediaDataExchange {
    async fn call(&self, operator_id: String, package: String, data: String) -> Result<HashMap<String, String>, String> {
        self.search_provider.search(&data)
    }
}
