/// The `Agent` structure is central to managing knowledge graphs within specific domains.
/// It represents an individual entity equipped with capabilities to construct, update,
/// and interrogate knowledge graphs, which are composed of nodes representing concepts
/// (words or phrases), with edges illustrating the relationships or dependencies among
/// these concepts.
///
/// # Key Functionalities
///
/// - `build_knowledge_graph`: Builds a knowledge graph from textual input by parsing
///   text to identify and establish nodes and their interconnections, thus structuring
///   the agent's domain understanding.
/// - `update_knowledge_graph`: Updates the knowledge graph with new information,
///   allowing for the incremental enrichment of the agent's knowledge base.
/// - `search`: Searches within the graph for nodes containing specific strings,
///   facilitating efficient information retrieval based on query terms.
/// - `summarize`: Generates a concise overview of the knowledge graph's content,
///   aiding in the visualization of the graph's structure.
///
/// # Direct Manipulation Methods
///
/// Methods are provided for the direct manipulation of the agent's domains, skills,
/// and knowledge, including adding or removing elements. This allows for more granular
/// control over the contents of the knowledge graph. Predicate methods are also provided
/// to check for the existence of specific domains, skills, or knowledge within the agent,
/// enhancing the utility for querying the agent's state.
///
/// # Serialization Support
///
/// Serialization support is incorporated through `to_json` and `from_json` methods,
/// enabling the conversion of an agent's state to and from a JSON format. This facilitates
/// easy storage, transmission, and reconstruction of the agent's knowledge graph, making
/// the module versatile for applications requiring data exchange and persistence.
///
/// # Overview
///
/// This module leverages Rust's powerful type system and ownership model to manage complex
/// data structures efficiently and safely, illustrating a practical application of graph
/// theory in knowledge representation and processing.



use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Agent<'a> {
    pub state: usize,
    pub q_table: Vec<Vec<f32>>,
    pub knowledge_graph: HashMap<&'a str, Vec<&'a str>>,
    pub domain: Vec<&'a str>,
    pub skills: Vec<&'a str>,
    pub provider_metadata: Vec<ProviderMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeAgent {
    knowledge_graph: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub name: String,
    pub provider_type: Vec<String>,
    pub supported_content_types: Vec<String>,
    pub cost_per_request: CostPerRequest,
    pub copyright_ownership: String,
    pub data_reproduction_rights: String,
    pub data_handling: DataHandling,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CostPerRequest {
    pub amount: f32,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataHandling {
    pub storage_duration: String,
    pub usage_policy: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub accuracy: f32,
    pub speed: String,
}

impl<'a> Agent<'a> {
    pub fn new() -> Self {
        Self {
            state: 0,
            q_table: Vec::new(),
            knowledge_graph: HashMap::new(),
            domain: Vec::new(),
            skills: Vec::new(),
            provider_metadata: Vec::new(),
        }
    }

    pub fn add_domain(&mut self, domain: &'a str) {
        self.domain.push(domain);
    }

    pub fn add_skill(&mut self, skill: &'a str) {
        self.skills.push(skill);
    }

    pub fn add_knowledge(&mut self, head_word: &'a str, dep_word: &'a str) {
        self.knowledge_graph
            .entry(head_word)
            .or_insert_with(Vec::new)
            .push(dep_word);
    }

    pub fn remove_domain(&mut self, domain: &str) {
        self.domain.retain(|&d| d != domain);
    }

    pub fn remove_skill(&mut self, skill: &str) {
        self.skills.retain(|&s| s != skill);
    }

    pub fn remove_knowledge(&mut self, head_word: &str, dep_word: &str) {
        if let Some(deps) = self.knowledge_graph.get_mut(head_word) {
            deps.retain(|&d| d != dep_word);
        }
    }

    pub fn has_domain(&self, domain: &str) -> bool {
        self.domain.contains(&domain)
    }

    pub fn has_skill(&self, skill: &str) -> bool {
        self.skills.contains(&skill)
    }

    pub fn has_knowledge(&self, head_word: &str, dep_word: &str) -> bool {
        self.knowledge_graph
            .get(head_word)
            .map_or(false, |deps| deps.contains(&dep_word))
    }

    pub fn get_domains(&self) -> Vec<&str> {
        self.domain.clone()
    }

    pub fn get_skills(&self) -> Vec<&str> {
        self.skills.clone()
    }

    pub fn get_knowledge(&self) -> Vec<(&str, &str)> {
        self.knowledge_graph
            .iter()
            .flat_map(|(&head_word, deps)| deps.iter().map(move |&dep_word| (head_word, dep_word)))
            .collect()
    }

    pub fn merge(&mut self, other: &Agent<'a>) {
        self.domain.extend(other.domain.iter().cloned());
        self.skills.extend(other.skills.iter().cloned());
        for (&head_word, deps) in &other.knowledge_graph {
            for &dep_word in deps {
                self.add_knowledge(head_word, dep_word);
            }
        }
        self.provider_metadata.extend(other.provider_metadata.clone());
    }

    pub fn clear(&mut self) {
        self.domain.clear();
        self.skills.clear();
        self.knowledge_graph.clear();
        self.provider_metadata.clear();
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn add_provider_metadata(&mut self, metadata: ProviderMetadata) {
        self.provider_metadata.push(metadata);
    }

    pub fn remove_provider_metadata(&mut self, name: &str) {
        self.provider_metadata.retain(|m| m.name != name);
    }

    pub fn get_provider_metadata(&self, name: &str) -> Option<&ProviderMetadata> {
        self.provider_metadata.iter().find(|m| m.name == name)
    }

    pub fn build_knowledge_graph(&mut self, text: &str) {
        // Implementation to build knowledge graph from text
        let words: Vec<&str> = text.split_whitespace().collect();
        for i in 0..words.len() {
            let head_word = words[i];
            self.knowledge_graph.entry(head_word).or_insert_with(Vec::new);
            if i + 1 < words.len() {
                let dep_word = words[i + 1];
                self.add_knowledge(head_word, dep_word);
            }
        }
    }
    
    pub fn update_knowledge_graph(&mut self, text: &str) {
        // Implementation to update the knowledge graph with new information from text
        let words: Vec<&str> = text.split_whitespace().collect();
        for i in 0..words.len() {
            let head_word = words[i];
            if !self.knowledge_graph.contains_key(head_word) {
                self.knowledge_graph.insert(head_word, Vec::new());
            }
            if i + 1 < words.len() {
                let dep_word = words[i + 1];
                if !self.has_knowledge(head_word, dep_word) {
                    self.add_knowledge(head_word, dep_word);
                }
            }
        }
    }
    
    pub fn search(&self, query: &str) -> Vec<&str> {
        // Implementation to search for nodes containing the query string
        self.knowledge_graph
            .iter()
            .filter(|(head_word, _)| head_word.contains(query))
            .map(|(head_word, _)| *head_word)
            .collect()
    }
    
    pub fn summarise(&self) -> String {
        // Implementation to generate a summary of the knowledge graph
        let mut summary = String::new();
        for (head_word, deps) in &self.knowledge_graph {
            summary.push_str(head_word);
            summary.push_str(": ");
            summary.push_str(&deps.join(", "));
            summary.push('\n');
        }
        summary
    }
}

impl KnowledgeAgent {
    pub fn new() -> Self {
        Self {
            knowledge_graph: HashMap::new(),
        }
    }

    pub fn update_knowledge_graph(&mut self, text: &str) {
        let words: Vec<&str> = text.split_whitespace().collect();
        for i in 0..words.len() {
            let head_word = words[i];
            if !self.knowledge_graph.contains_key(head_word) {
                self.knowledge_graph.insert(head_word.to_string(), Vec::new());
            }
            if i + 1 < words.len() {
                let dep_word = words[i + 1];
                if !self.knowledge_graph[head_word].contains(&dep_word.to_string()) {
                    self.knowledge_graph
                        .get_mut(head_word)
                        .unwrap()
                        .push(dep_word.to_string());
                }
            }
        }
    }

    pub fn search(&self, query: &str) -> Vec<&str> {
        self.knowledge_graph
            .iter()
            .filter(|(head_word, _)| head_word.contains(query))
            .flat_map(|(_, deps)| deps.iter().map(|dep| dep.as_str()))
            .collect()
    }
}