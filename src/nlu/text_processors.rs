//! # Text Processors
//!
//! This module provides a collection of text processing utilities for natural language understanding (NLU).
//!
//! ## Functionality
//!
//! The module offers the following text processing capabilities:
//!
//! - Tokenization: Breaking down text into individual tokens (words, punctuation, etc.).
//! - Part-of-Speech (POS) Tagging: Assigning grammatical categories to each token.
//! - Named Entity Recognition (NER): Identifying and classifying named entities in the text.
//! - Dependency Parsing: Analyzing the grammatical structure and relationships between tokens.
//! - Sentiment Analysis: Determining the sentiment (positive, negative, or neutral) of the text.
//! - Language Detection: Identifying the language of the input text.
//! - Text Normalization: Converting text to a standardized format (e.g., lowercase, removing punctuation).
//! - Stop Word Removal: Filtering out common words that do not carry significant meaning.
//! - Stemming and Lemmatization: Reducing words to their base or dictionary form.
//!
//! ## Usage
//!
//! To use the text processors, follow these steps:
//!
//! 1. Create an instance of the desired text processor (e.g., `Tokenizer`, `POSTagger`, `NERTagger`).
//! 2. Provide the input text to the processor's corresponding method (e.g., `tokenize`, `tag_pos`, `recognize_entities`).
//! 3. Retrieve the processed output, which will be in the form of a structured representation (e.g., `Vec<Token>`, `Vec<(String, String)>`).
//!
//! Here's an example of using the tokenizer:
//!
//! ```rust
//! use text_processors::Tokenizer;
//!
//! let text = "This is a sample sentence.";
//! let tokenizer = Tokenizer::new();
//! let tokens = tokenizer.tokenize(text);
//! println!("{:?}", tokens);
//! ```
//!
//! ## Customisation
//!
//! The text processors provide default models and configurations for each processing task. However, you can customize the behavior by:
//!
//! - Providing your own trained models for specific languages or domains.
//! - Adjusting the configuration parameters of the processors (e.g., setting the language, enabling/disabling certain features).
//! - Implementing your own text processing traits and structs to extend or modify the functionality.
//!
//! ## Performance
//!
//! The text processors are designed to be efficient and handle large volumes of text. However, the performance may vary depending on the complexity of the input text and the chosen processing tasks.
//!
//! To optimize performance, consider the following:
//!
//! - Preprocess and clean the input text before applying the processors.
//! - Use appropriate data structures and algorithms for efficient processing.
//! - Parallelize the processing tasks when working with large datasets.
//! - Cache the results of expensive operations if they are reused frequently.
//!
//! ## Dependencies
//!
//! The text processors module relies on the following dependencies:
//!
//! - `nltk` (Natural Language Toolkit): A popular Python library for natural language processing.
//! - `spacy`: A fast and powerful library for advanced natural language processing tasks.
//! - `rust-bert`: A Rust library for working with BERT (Bidirectional Encoder Representations from Transformers) models.
//!

use crate::bindings::spacy_bindings::{Doc, EntityGraph, EntityLabel, LangModel, SpacyModule, Token, TokenPos};
use std::collections::HashMap;

pub struct TextProcessor {
    pipeline: Vec<Box<dyn Fn(&str, &str) -> String>>,
}

impl TextProcessor {
    pub fn new(pipeline: Vec<Box<dyn Fn(&str, &str) -> String>>) -> TextProcessor {
        TextProcessor { pipeline }
    }

    pub async fn process_text(&self, text: &str, classification: &str) -> HashMap<String, Vec<String>> {
        let mut result = HashMap::new();
        let original_text = text.to_string();
        let processed_text = self.process_pipeline(text, classification);
        let classification = classification.to_string();

        let doc = SpacyModule::model_default().nlp(&processed_text).await.unwrap();
        let python = Python::acquire_gil().python();
        let named_entities = doc.named_entities(python).unwrap();
        let lemmas = doc.lemmas(python).unwrap();

        result.insert("Original Text".to_string(), vec![original_text]);
        result.insert("Processed Text".to_string(), vec![processed_text]);
        result.insert("Classification".to_string(), vec![classification]);
        result.insert("Named Entities".to_string(), named_entities);
        result.insert("Lemmas".to_string(), lemmas);

        for step in &self.pipeline {
            let step_name = get_function_name(step);
            let step_result = step(&processed_text, classification);
            result.insert(step_name, vec![step_result]);
        }

        result
    }

    pub async fn process_text_all_permutations(
        &self,
        text: &str,
        classification: &str,
    ) -> Vec<HashMap<String, Vec<String>>> {
        let mut results = Vec::new();
        let original_text = text.to_string();
        let classification = classification.to_string();
        let permutations = get_text_processing_permutations(&self.pipeline);

        for (i, permutation) in permutations.iter().enumerate() {
            let mut result = HashMap::new();
            let permutation_name = format!("Permutation {}", i + 1);
            let processed_text = self.process_pipeline_permutation(text, classification, permutation);

            let doc = SpacyModule::model_default().nlp(&processed_text).await.unwrap();
            let python = Python::acquire_gil().python();
            let named_entities = doc.named_entities(python).unwrap();
            let lemmas = doc.lemmas(python).unwrap();

            result.insert("Original Text".to_string(), vec![original_text.clone()]);
            result.insert("Processed Text".to_string(), vec![processed_text]);
            result.insert("Classification".to_string(), vec![classification.clone()]);
            result.insert("Named Entities".to_string(), named_entities);
            result.insert("Lemmas".to_string(), lemmas);

            for step in permutation {
                let step_name = get_function_name(step);
                let step_result = step(&processed_text, classification);
                result.insert(step_name, vec![step_result]);
            }

            results.push(result);
        }

        results
    }

    fn process_pipeline(&self, text: &str, classification: &str) -> String {
        let mut processed_text = text.to_string();
        for step in &self.pipeline {
            processed_text = step(&processed_text, classification);
        }
        processed_text
    }

    fn process_pipeline_permutation(
        &self,
        text: &str,
        classification: &str,
        permutation: &[Box<dyn Fn(&str, &str) -> String>],
    ) -> String {
        let mut processed_text = text.to_string();
        for step in permutation {
            processed_text = step(&processed_text, classification);
        }
        processed_text
    }
}

pub fn get_text_processing_permutations(
    pipeline: &[Box<dyn Fn(&str, &str) -> String>],
) -> Vec<Vec<Box<dyn Fn(&str, &str) -> String>>> {
    if pipeline.is_empty() {
        return vec![vec![]];
    }

    let mut permutations = Vec::new();
    for (i, step) in pipeline.iter().enumerate() {
        let remaining_steps = pipeline
            .iter()
            .enumerate()
            .filter(|&(j, _)| i != j)
            .map(|(_, s)| s)
            .cloned()
            .collect::<Vec<_>>();
        let sub_permutations = get_text_processing_permutations(&remaining_steps);
        for sub_permutation in sub_permutations {
            let mut permutation = sub_permutation;
            permutation.insert(0, step.clone());
            permutations.push(permutation);
        }
    }

    permutations
}

fn get_function_name(function: &Box<dyn Fn(&str, &str) -> String>) -> String {
    // Implement a way to get the function name as a string
    // You can use a custom trait or any other suitable method
    // For simplicity, let's assume the function name is "Step"
    "Step".to_string()
}

fn process_message(
    delegate: &Delegate,
    knowledge_agent: &KnowledgeAgent,
    q_learning_agent: &QLearningAgent,
    data: &HashMap<String, String>,
) -> String {
    // Use the delegate to extract relevant information from the data
    let interests = delegate
        .attributes
        .get("interests")
        .map(|attr| attr.values.iter().cloned().collect::<Vec<String>>())
        .unwrap_or_default();
    let expertise = delegate
        .attributes
        .get("expertise")
        .map(|attr| attr.values.iter().cloned().collect::<Vec<String>>())
        .unwrap_or_default();

    // Use the Spacy bindings to process the text data
    let text_input = data.values().cloned().collect::<Vec<String>>().join(" ");
    let doc = SpacyModule::model_default().nlp(text_input).await.unwrap();
    let python = Python::acquire_gil().python();
    // Extract named entities and their positions
    let named_entities = doc.named_entities(python).unwrap();
    // Get the lemmatized form of each token
    let lemmas = doc.lemmas(python).unwrap();

    // Use the knowledge agent to search for relevant information in the knowledge graph
    let relevant_info = knowledge_agent
        .search(&interests.join(" "))
        .into_iter()
        .chain(knowledge_agent.search(&expertise.join(" ")))
        .collect::<Vec<&str>>();

    // Use the Q-learning agent to select the best action based on the current state
    let state = q_learning_agent.get_state(data);
    let action = q_learning_agent.get_best_action(state);

    // Generate a response based on the selected action and relevant information
    format!(
        "Based on your interests in {} and expertise in {}, I suggest you {}. Here's some relevant information: {}. Named entities: {:?}. Lemmas: {:?}",
        interests.join(", "),
        expertise.join(", "),
        action,
        relevant_info.join(", "),
        named_entities,
        lemmas
    )
}