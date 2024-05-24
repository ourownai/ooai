//! This module defines a `MessageFilter` struct that can be used to filter and paraphrase text messages.
//! The struct contains a blacklist of prohibited terms with similarity scores, a whitelist of allowed terms,
//! and a language model from the Spacy library.
//!
//! The `MessageFilter` struct has several methods:
//!
//! - `new`: Constructs a new `MessageFilter` instance given a blacklist, a whitelist, and a language model.
//!
//! - `filter_message`: Filters a message by replacing blacklisted words and their similar tokens with asterisks
//!   (`*****`), based on a given similarity threshold.
//!
//! - `paraphrase_message`: Paraphrases a message by replacing blacklisted words with similar tokens while keeping
//!   the allowed terms unchanged.
//!
//! - `get_similar_tokens`: Returns a `HashMap` of tokens similar to the blacklisted terms, based on a similarity
//!   threshold.
//!
//! - `mask_tokens`: Replaces blacklisted terms and their similar tokens with asterisks (`*****`) in a message,
//!   given a similarity threshold.
//!
//! - `paraphrase_tokens`: Replaces blacklisted terms in a message with similar tokens while keeping the allowed
//!   terms unchanged.
//!
//! - `get_similar_token`: Returns a similar token to a given token, based on a similarity threshold.
//!
//! In the `main` function, a blacklist and whitelist are created, and a `MessageFilter` instance is constructed
//! with these lists and a language model. Then, three messages are filtered using the `filter_message` method
//! with a similarity threshold of 0.8. The code checks the filtered messages against the expected results using
//! `assert_eq!`.

use std::collections::{HashMap, HashSet};
use spacy::Spacy;

pub struct MessageFilter {
    blacklist: HashMap<String, f32>,
    whitelist: HashSet<String>,
    spacy: Spacy,
}

impl MessageFilter {
    pub fn new(blacklist: HashMap<String, f32>, whitelist: HashSet<String>, lang: &str) -> Self {
        let spacy = Spacy::new(lang).unwrap(); // Initialize Spacy language model
        MessageFilter {
            blacklist,
            whitelist,
            spacy,
        }
    }

    pub fn filter_message(&self, message: &str, threshold: f32, lang: &str) -> Option<String> {
        let similar_tokens = self.get_similar_tokens();
        let masked_tokens = self.mask_tokens(message, &similar_tokens, threshold, lang);
        let masked_message = masked_tokens.join(" ");
        if masked_message != message {
            Some(masked_message)
        } else {
            None
        }
    }

    pub fn paraphrase_message(&self, message: &str, lang: &str) -> Option<String> {
        let paraphrased_tokens = self.paraphrase_tokens(message, lang);
        let paraphrased_message = paraphrased_tokens.join(" ");
        if paraphrased_message != message {
            Some(format!(
                "{} I'm sorry I was not able to provide more information on this occasion.",
                paraphrased_message
            ))
        } else {
            None
        }
    }

    fn get_similar_tokens(&self) -> HashMap<String, f32> {
        let mut similar_tokens = HashMap::new();
        let similar_token_threshold = 0.8; // Threshold for similarity score
        for (token, _) in &self.blacklist {
            let doc = self.spacy.run(token);
            for ent in doc.ents() {
                for ent_token in ent.tokens() {
                    let similarity = self.spacy.similarity(ent_token.text(), token);
                    if similarity >= similar_token_threshold {
                        similar_tokens.insert(ent_token.text().to_owned(), similarity);
                    }
                }
            }
        }
        similar_tokens
    }

    fn mask_tokens(
        &self,
        message: &str,
        similar_tokens: &HashMap<String, f32>,
        threshold: f32,
        lang: &str,
    ) -> Vec<String> {
        let doc = self.spacy.run_with_language(message, lang).unwrap();
        let mut masked_tokens = vec![];
        for token in doc.tokens() {
            let token_text = token.text();
            let token_lower = token_text.to_lowercase();
            if let Some(score) = similar_tokens.get(&token_lower) {
                if *score >= threshold {
                    masked_tokens.push("*****");
                    continue;
                }
            }
            if self.whitelist.contains(&token_lower) {
                masked_tokens.push(token_text);
            } else if self.blacklist.contains_key(&token_lower) {
                masked_tokens.push("*****");
            } else {
                masked_tokens.push(token_text);
            }
        }
        masked_tokens
    }

    fn paraphrase_tokens(&self, message: &str, lang: &str) -> Vec<String> {
        let doc = self.spacy.run_with_language(message, lang).unwrap();
        let mut paraphrased_tokens = vec![];
        for token in doc.tokens() {
            let token_text = token.text();
            let token_lower = token_text.to_lowercase();
            if self.whitelist.contains(&token_lower) {
                paraphrased_tokens.push(token_text);
            } else if let Some(similar_token) = self.get_similar_token(&token_lower) {
                paraphrased_tokens.push(similar_token);
            } else {
                paraphrased_tokens.push(token_text);
            }
        }
        paraphrased_tokens
    }

    fn get_similar_token(&self, token: &str) -> Option<String> {
        let similar_token_threshold = 0.8; // Threshold for similarity score
        for (blacklisted_token, _) in &self.blacklist {
            let similarity = self.spacy.similarity(token, blacklisted_token);
            if similarity >= similar_token_threshold {
                return Some(blacklisted_token.to_owned());
            }
        }
        None
    }
}

fn main() {
    let blacklist = [
        ("personal information".to_owned(), 0.8),
        ("password".to_owned(), 0.9),
        ("credit card".to_owned(), 0.7),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<String, f32>>();
    let whitelist = [
        "balance".to_owned(),
        "favorite".to_owned(),
        "color".to_owned(),
    ]
    .iter()
    .cloned()
    .collect::<HashSet<String>>();
    let message_filter = MessageFilter::new(blacklist, whitelist, "en_core_web_sm");
    let message1 = "Can you tell me my credit card balance?";
    let message2 = "What's my favorite color?";
    let message3 = "What is your favorite book?";
    let threshold = 0.8;
    let response1 = message_filter.filter_message(message1, threshold, "en");
    let response2 = message_filter.filter_message(message2, threshold, "en");
    let response3 = message_filter.filter_message(message3, threshold, "en");
    assert_eq!(
        response1,
        Some("Can you tell me my ***** *****?".to_owned())
    );
    assert_eq!(response2, Some("What's my favorite color?".to_owned()));
    assert_eq!(
        response3,
        Some("What is your favorite *****?".to_owned())
    );
}