//! This Rust module defines a TextProcessor struct that can be used to process text data.
//! The struct has two methods, process_text and process_text_all_permutations, which return
//! a Graph object that represents the processing pipeline.
//!
//! The process_text method takes a &str argument text and a &str argument classification,
//! representing the text to be processed and the classification of the text, respectively.
//! It returns a Graph object that represents the processing pipeline, where each node in the
//! graph represents a step in the pipeline and each edge in the graph represents the data flow
//! between the steps.
//!
//! The process_text_all_permutations method is similar to process_text, but it generates a
//! Graph object for every possible permutation of the processing steps.
//!
//! The TextProcessor struct has a pipeline field, which is a vector of function objects that
//! take two &str arguments and return a String. These functions represent the processing steps
//! that will be applied to the text.
//!
//! The TextProcessor struct also defines a private method, process_pipeline, which takes a &str
//! argument text and a &str argument classification, and applies the processing steps in the
//! pipeline field to the text.
//!
//! The module also defines several utility functions, including detect_language, tokenize_text,
//! lemmatize_text, remove_stopwords, and stem_text. These functions are used as processing steps
//! in the pipeline field. Each function takes a &Language object, which represents the language
//! of the text, a &str argument text, a &str argument classification, and a &mut Graph object,
//! which is used to construct the processing pipeline. The functions perform various operations
//! on the text, such as tokenization, lemmatization, stopword removal, and stemming, and add
//! nodes and edges to the graph to represent the processing steps.

use petgraph::graph::{Graph, NodeIndex};
use std::collections::HashMap;
use text_processor::{TextProcessor, Language};

pub fn detect_language(language: &Language, text: &str) -> String {
    let doc = language.tokenizer(text);
    let mut languages = HashMap::new();
    for token in doc {
        let lang = language.detect_language(&token.text);
        *languages.entry(lang).or_insert(0) += 1;
    }
    languages.into_iter().max_by_key(|(_, count)| *count).unwrap().0.to_string()
}

pub fn tokenize_text(language: &Language, text: &str) -> Vec<String> {
    let doc = language.tokenizer(text);
    doc.into_iter().map(|token| token.text).collect()
}

pub fn lemmatize_text(language: &Language, tokens: &[String]) -> Vec<String> {
    tokens.iter().map(|token| language.lemmatizer(token)).collect()
}

pub fn remove_stopwords(language: &Language, tokens: &[String]) -> Vec<String> {
    tokens.iter().filter(|token| !language.is_stop(token)).cloned().collect()
}

pub fn stem_text(language: &Language, tokens: &[String]) -> Vec<String> {
    tokens.iter().map(|token| language.stemmer(token)).collect()
}

pub struct TextProcessingPipeline {
    steps: Vec<Box<dyn Fn(&Language, &str) -> Vec<String>>>,
}

impl TextProcessingPipeline {
    pub fn new(steps: Vec<Box<dyn Fn(&Language, &str) -> Vec<String>>>) -> Self {
        TextProcessingPipeline { steps }
    }

    pub fn process(&self, language: &Language, text: &str) -> Vec<String> {
        let mut tokens = vec![text.to_string()];
        for step in &self.steps {
            tokens = step(language, &tokens.join(" "));
        }
        tokens
    }
}

fn main() {
    let language = Language::new("en_core_web_sm").unwrap();
    let pipeline = TextProcessingPipeline::new(vec![
        Box::new(|language, text| vec![detect_language(language, text)]),
        Box::new(tokenize_text),
        Box::new(lemmatize_text),
        Box::new(remove_stopwords),
        Box::new(stem_text),
    ]);

    let text = "The quick brown fox jumps over the lazy dog.";
    let processed_tokens = pipeline.process(&language, text);
    println!("Processed tokens: {:?}", processed_tokens);

    let graph = Graph::<String, &str>::new();
    let mut node_map = HashMap::new();
    for (i, token) in processed_tokens.iter().enumerate() {
        let node = graph.add_node(token.clone());
        node_map.insert(i, node);
    }
    for i in 0..processed_tokens.len() - 1 {
        let source = node_map[&i];
        let target = node_map[&(i + 1)];
        graph.add_edge(source, target, "next");
    }
    println!("Graph: {:#?}", graph);
}
