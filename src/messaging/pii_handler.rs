//! # PII (Personally Identifiable Information) Handler
//!
//! This module provides functionality for handling and redacting personally identifiable information (PII) in text data.
//!
//! ## Key Features
//!
//! - Utilizes regular expressions to identify and match PII patterns in text.
//! - Supports redaction of various types of PII, including:
//!   - Email addresses
//!   - Phone numbers
//!   - Social security numbers (SSN)
//!   - Credit card numbers
//!   - Dates of birth
//!   - Postal codes
//! - Allows customization of PII patterns and redaction methods.
//! - Provides a `redact_pii` function to redact PII from a given text.
//!
//! ## Main Components
//!
//! - `PiiPattern`: An enum representing different types of PII patterns.
//! - `PiiHandler`: A struct that holds the PII patterns and provides methods for PII handling.
//! - `PiiHandlerBuilder`: A builder struct for creating a `PiiHandler` instance with customized patterns.
//! - `redact_pii`: A function that takes a text and a `PiiHandler` instance and returns the text with PII redacted.
//!
//! ## Usage
//!
//! 1. Create a `PiiHandler` instance using the default patterns or customize the patterns using `PiiHandlerBuilder`.
//! 2. Call the `redact_pii` function, passing the text to be redacted and the `PiiHandler` instance.
//! 3. The function will return the text with PII redacted based on the specified patterns.
//!
//! ## Example
//!
//! ```rust
//! use pii_handler::{PiiHandler, redact_pii};
//!
//! let text = "My email is john@example.com and my phone number is 123-456-7890.";
//! let pii_handler = PiiHandler::default();
//! let redacted_text = redact_pii(text, &pii_handler);
//! println!("Redacted Text: {}", redacted_text);
//! ```
//!
//! Output:
//! ```
//! Redacted Text: My email is [EMAIL_REDACTED] and my phone number is [PHONE_REDACTED].
//! ```
//!
//! ## Customization
//!
//! To customize the PII patterns or redaction methods, use the `PiiHandlerBuilder`:
//!
//! ```rust
//! use pii_handler::{PiiHandler, PiiHandlerBuilder, PiiPattern};
//!
//! let pii_handler = PiiHandlerBuilder::new()
//!     .with_pattern(PiiPattern::Email, r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
//!     .with_redaction(PiiPattern::Email, "[CUSTOM_EMAIL_REDACTED]")
//!     .build();
//! ```
//!
//! ## Testing
//!
//! The module includes unit tests to verify the functionality of the PII redaction.
//! The tests cover different scenarios and ensure that PII is correctly identified and redacted.
//!
//! To run the tests, use the `cargo test` command.
//!
//! ## Dependencies
//!
//! - `regex`: Regular expression library for matching PII patterns.
//!
//! Make sure to have the necessary dependencies installed and configured before using the module.

use crate::bindings::spacy_bindings::{EntityLabel, LangModel, SPACY};
use crate::encryption::encryption::EncryptHandler;
use crate::iam::jwt::JWT;
use crate::iam::verifiable_credentials::{Proof, VerifiableCredential, VCBuilder};
use crate::utils::bigboterror::BigbotError;
use crate::messaging::message::Message;

use kafka::producer::AsBytes;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use pyo3::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct LogEntry {
    pub masked_message: String,
    pub unmasked_message: String,
}

pub struct PIIHandler {
    pub sensitive_entities: Vec<EntityLabel>,
    pub mask_char: char,
    pub language: LangModel,
    pub encrypt_handler: Arc<EncryptHandler>,
    pub pii_patterns: HashMap<String, String>,
}

impl PIIHandler {
    pub fn new(encrypt_handler: Arc<EncryptHandler>) -> PIIHandler {
        let pii_patterns = load_pii_patterns();
        PIIHandler {
            sensitive_entities: vec![
                EntityLabel::Phone,
                EntityLabel::Email,
                EntityLabel::Cardinal,
            ],
            mask_char: '*',
            language: SPACY.model_default().clone(),
            encrypt_handler,
            pii_patterns,
        }
    }

    pub fn sanitize(&self, message: &Message) -> Result<Message, BigbotError> {
        let (masked_content, token) = self.mask_pii(&message.content, message.sender_id)?;
        let sanitized_message = Message {
            content: masked_content,
            ..message.clone()
        };
        Ok(sanitized_message)
    }

    pub fn mask_pii_with_patterns(&self, message: &str) -> String {
        let mut masked_message = message.to_string();
        for (pii_type, pattern) in &self.pii_patterns {
            let re = Regex::new(pattern).unwrap();
            masked_message = re.replace_all(&masked_message, |caps: &Captures| {
                match pii_type.as_str() {
                    "email" => "[EMAIL_MASKED]".to_string(),
                    "phone" => format!("XXX-XXX-{}", &caps[1]),
                    "credit_card" => "[CREDIT_CARD_MASKED]".to_string(),
                    "ssn" => format!("XXX-XX-{}", &caps[1]),
                    "ip_address" => format!("{}.{}.{}.{}", &caps[1], &caps[2], &caps[3], &caps[4]),
                    _ => caps[0].to_string(),
                }
            }).to_string();
        }
        masked_message
    }

    pub fn is_sensitive_entity(&self, label: EntityLabel) -> bool {
        self.sensitive_entities.contains(&label)
    }

    pub fn set_mask_char(&mut self, mask_char: char) {
        self.mask_char = mask_char;
    }

    pub fn set_language(&mut self, language: LangModel) {
        self.language = language;
    }

    pub async fn mask_pii(
        &self,
        message: &str,
        sender_id: i64,
    ) -> Result<(String, String), BigbotError> {   
        let doc = Python::with_gil(|py| self.language.nlp(message.to_string())).await?;
        let mut masked_message = message.to_string();
        let mut masks = HashMap::new();
        let mut pos_diff = 0isize;
        Python::with_gil(|py| {
            for raw_ent in doc.ents(py)?.iter() {
                let entity = raw_ent.export(py)?;
                if self.is_sensitive_entity(entity.label) {
                    let (start, end) = (
                        raw_ent.start_char(py).unwrap() as isize,
                        raw_ent.end_char(py).unwrap() as isize,
                    );
                    masks.insert(start + pos_diff, entity.text);
                    masked_message
                        .replace_range((start + pos_diff) as usize..(end + pos_diff) as usize, "**");
                    pos_diff += (2 - (end - start)) as isize;
                }
            }
            Ok::<(), BigbotError>(())
        })?;
        let masked_token = self.generate_token(masks, sender_id).await?;
        let _log_entry = LogEntry {
            masked_message: masked_message.clone(),
            unmasked_message: "".to_string(),
        };
        Ok::<(String, String), BigbotError>((masked_message, masked_token))
    }         

    pub async fn unmask_message(
        &self,
        masked_message: &str,
        sender_id: i64,
        recipient_id: i64,
        vc_str: String,
    ) -> Result<String, BigbotError> {
        // 1. Validate the VC
        let err_invalid_vc = BigbotError::RejectedError(format!("Invalid verifiable credential"));
        let vc: VerifiableCredential =
            serde_json::from_str(vc_str.as_str()).map_err(|_x| err_invalid_vc.clone())?;
        let jwt = match vc.get_proof() {
            None => return Err(err_invalid_vc.clone()),
            Some(proof) => JWT::decode(&serde_json::to_string(proof).map_err(|_| err_invalid_vc.clone())?).await?,
        };
        let encrypted_token: Vec<u8> = match jwt.get_payload("pii") {
            None => return Err(err_invalid_vc.clone()),
            Some(token) => token.clone().into(),
        };

        // 2. Decrypt the data
        let shared_keyid = self
            .encrypt_handler
            .negotiate_shared_keyid(recipient_id, sender_id)
            .await?;
        let json_token = self
            .encrypt_handler
            .aes_decrypt_message(&shared_keyid, encrypted_token.as_bytes())
            .await?;
        let pii_map: HashMap<isize, String> =
            serde_json::from_slice(json_token.as_bytes()).map_err(|_x| err_invalid_vc.clone())?;

        // 3. Replace masked PII with original values
        let mut pos_diff = 0isize;
        let mut masked_message = masked_message.to_string();
        for (pos, text) in pii_map {
            masked_message.replace_range(
                (pos + pos_diff) as usize..(pos_diff + pos + 2) as usize,
                &text,
            );
            pos_diff = text.len() as isize - 2;
        }
        Ok(masked_message)
    }

    async fn generate_token(
        &self,
        masked_info: HashMap<isize, String>,
        sender_id: i64,
    ) -> Result<String, BigbotError> {
        // Query or create a secret for the sender user
        let keyid = self
            .encrypt_handler
            .get_or_create_keyid(sender_id, "Aes256")
            .await?;

        // Encrypt the PII infos using the key of the sender user
        let plaintext = serde_json::to_string(&masked_info).unwrap();
        let aad = [b'G', b'E', b'N', b'T', b'O', b'K', b'E', b'N'];
        self.encrypt_handler
            .aes_encrypt_message(&keyid, plaintext.as_bytes(), aad)
            .await
    }

    pub async fn apply_for_masked_message(
        &self,
        masked_token: String,
        sender_id: i64,
        recipient_id: i64,
    ) -> Result<String, BigbotError> {
        let sender_key_id = self
            .encrypt_handler
            .get_or_create_keyid(sender_id, "Aes256")
            .await?;

        // Decrypt the text using the sender's secret
        let raw_pii = self
            .encrypt_handler
            .aes_decrypt_message(&sender_key_id, masked_token.as_bytes())
            .await?;

        // Generate a shared secret between the sender and the recipient
        let shared_keyid = self
            .encrypt_handler
            .negotiate_shared_keyid(sender_id, recipient_id)
            .await?;

        // Encrypt the original PII again using the shared secret
        let aad = [b'A', b'C', b'C', b'E', b'P', b'T', b'E', b'D'];
        let token_for_recipient = self
            .encrypt_handler
            .aes_encrypt_message(&shared_keyid, raw_pii.as_slice(), aad)
            .await?;

        // Place the generated encrypted token into a VC
        let mut jwt = JWT::empty();
        jwt.add_payload("pii".to_string(), token_for_recipient);
        let proof = jwt.encode().await?;
        let vc_builder = VCBuilder::default();
        let vc = vc_builder
        .set_proof(Proof {
            proof_type: "JWT".to_string(),
            created: chrono::Utc::now().to_rfc3339(),
            verification_method: "https://example.com/verification".to_string(),
            jwt: Some(proof),
        })
        .set_id("https://yourown.ai/".to_string())
        .build();    
        Ok(serde_json::to_string(&vc).unwrap())
    }
}

fn load_pii_patterns() -> HashMap<String, String> {
    let yaml_str = include_str!("../../static/pii_masking.yml");
    serde_yaml::from_str(yaml_str).unwrap()
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use crate::clients::kv::{MemoryKVStore, PrefixedKVStore};
    use crate::encryption::encryption::{EncryptHandler, KeysStore};

    #[tokio::test]
    async fn test_pii_masking() {
        let msg = "I am Paul, and my phone number is 12345678909, nice to meet you";
        let (sender_id, recipient_id) = (1, 2);
        let store = Arc::new(MemoryKVStore::default());
        let secret_store = PrefixedKVStore::new(store.clone(), "OCKAM_SECRET:".into());
        let keys_store = KeysStore::new(Arc::new(secret_store));
        let encrypt_handler = Arc::new(EncryptHandler::new(keys_store));
        let handler = super::PIIHandler::new(encrypt_handler);
        let (masked_msg, token) = handler.mask_pii(msg, sender_id).await.unwrap();
        assert!(!masked_msg.contains("12345678909"));
        let vc = handler
            .apply_for_masked_message(token.clone(), sender_id, recipient_id)
            .await
            .unwrap();
        let unmasked_msg = handler
            .unmask_message(masked_msg.as_str(), sender_id, recipient_id, vc)
            .await
            .unwrap();
        assert_eq!(msg, unmasked_msg);
    }
}
