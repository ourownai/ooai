use crate::clients::kv::KVStore;
use crate::utils::bigboterror;

use async_trait::async_trait;
use base64::Engine;
use kafka::producer::AsBytes;
use rand::{thread_rng, RngCore};
use std::sync::Arc;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::Aead;
use x25519_dalek::{EphemeralSecret, PublicKey};

pub struct EncryptHandler {
    keyid_store: Arc<dyn KVStore>,
}

pub struct KeysStore {
    store: Arc<dyn KVStore>,
}

impl KeysStore {
    pub fn new(store: Arc<dyn KVStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl KVStore for KeysStore {
    async fn set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), bigboterror::BigbotError> {
        self.store
            .set(key, value)
            .await
            .map_err(|x| bigboterror::BigbotError::DatabaseError(format!("Failed to set key-value pair: {}", x)))
    }

    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, bigboterror::BigbotError> {
        self.store
            .get(key)
            .await
            .map_err(|x| bigboterror::BigbotError::DatabaseError(format!("Failed to get value: {}", x)))
    }

    async fn delete(&self, key: &[u8]) -> Result<(), bigboterror::BigbotError> {
        self.store
            .delete(key)
            .await
            .map_err(|x| bigboterror::BigbotError::DatabaseError(format!("Failed to delete key: {}", x)))
    }

    async fn keys(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>, bigboterror::BigbotError> {
        self.store
            .keys(prefix)
            .await
            .map_err(|x| bigboterror::BigbotError::DatabaseError(format!("Failed to get keys: {}", x)))
    }
}

impl EncryptHandler {
    pub fn new(keyid_store: Arc<dyn KVStore>) -> Self {
        Self { keyid_store }
    }

    pub(crate) async fn get_or_create_keyid(
        &self,
        user_id: i64,
        key_type: &str,
    ) -> Result<Vec<u8>, bigboterror::BigbotError> {
        let id: Vec<u8> = format!("{}:{}", key_type, user_id).into();
        match self.keyid_store.get(id.as_bytes()).await.map_err(|x| bigboterror::BigbotError::DatabaseError(format!("Failed to get value: {}", x)))? {
            Some(kid) => Ok(kid),
            None => {
                let keyid = generate_random_key();
                self.keyid_store.set(id, keyid.clone()).await.map_err(|x| bigboterror::BigbotError::DatabaseError(format!("Failed to set key-value pair: {}", x)))?;
                Ok(keyid)
            }
        }
    }

    pub(crate) async fn negotiate_shared_keyid(
        &self,
        user1: i64,
        user2: i64,
    ) -> Result<Vec<u8>, bigboterror::BigbotError> {
        let keyid1 = self.get_or_create_keyid(user1, "X25519").await.map_err(|e| bigboterror::BigbotError::DatabaseError(format!("Failed to get or create keyid: {}", e)))?;
        let keyid2 = self.get_or_create_keyid(user2, "X25519").await.map_err(|e| bigboterror::BigbotError::DatabaseError(format!("Failed to get or create keyid: {}", e)))?;
        let shared_secret = diffie_hellman(&keyid1, &keyid2);
        let shared_keyid = generate_aes_key(shared_secret.as_bytes());
        Ok(shared_keyid)
    }

    pub(crate) async fn aes_encrypt_message(
        &self,
        keyid: &[u8],
        plaintext: &[u8],
        aad: [u8; 8],
    ) -> Result<String, bigboterror::BigbotError> {
        let nonce: Vec<u8> = (0..12).map(|_x| thread_rng().next_u32() as u8).collect();
        let ciphertext = aes_gcm_encrypt(keyid, plaintext, nonce.as_slice(), aad.as_slice());
        let mut buf = Vec::with_capacity(ciphertext.len() + 20);
        buf.extend_from_slice(nonce.as_slice());
        buf.extend_from_slice(aad.as_slice());
        buf.extend_from_slice(ciphertext.as_slice());
        Ok(base64::engine::general_purpose::STANDARD.encode(buf))
    }

    pub(crate) async fn aes_decrypt_message(
        &self,
        keyid: &[u8],
        data: &[u8],
    ) -> Result<Vec<u8>, bigboterror::BigbotError> {
        let masked_token_bin = base64::engine::general_purpose::STANDARD.decode(data).unwrap();
        if masked_token_bin.len() < 20 {
            return Err(bigboterror::BigbotError::InvalidInput("invalid signature".into()));
        }

        let nonce = &masked_token_bin[..12];
        let aad = &masked_token_bin[12..20];
        let ciphertext = &masked_token_bin[20..];
        let plaintext = aes_gcm_decrypt(keyid, ciphertext, nonce, aad);
        Ok(plaintext)
    }

    pub(crate) async fn encrypt_message_for_users(
        &self,
        uids: Vec<i64>,
        plaintext: &[u8],
        aad: [u8; 8],
    ) -> Result<Vec<String>, bigboterror::BigbotError> {
        let mut encrypted_messages = vec![];
        for i in 0..uids.len() {
            for j in (i + 1)..uids.len() {
                let keyid = self.negotiate_shared_keyid(uids[i], uids[j]).await?;
                let encrypted_msg = self.aes_encrypt_message(&keyid, plaintext, aad).await?;
                encrypted_messages.push(encrypted_msg);
            }
        }
        Ok(encrypted_messages)
    }

    pub(crate) async fn decrypt_message_with_shared_key(
        &self,
        user1: i64,
        user2: i64,
        data: &[u8],
    ) -> Result<Vec<u8>, bigboterror::BigbotError> {
        let shared_keyid = self.negotiate_shared_keyid(user1, user2).await?;
        let decrypted_msg = self.aes_decrypt_message(&shared_keyid, data).await?;
        Ok(decrypted_msg)
    }
}

// Helper functions
fn generate_random_key() -> Vec<u8> {
    let mut rng = thread_rng();
    let mut key = vec![0u8; 32];
    rng.fill_bytes(&mut key);
    key
}

fn diffie_hellman(private_key: &[u8], public_key: &[u8]) -> Vec<u8> {
    let secret = EphemeralSecret::from_slice(private_key).unwrap();
    let public = PublicKey::from_slice(public_key).unwrap();
    let shared_secret = secret.diffie_hellman(&public);
    shared_secret.as_bytes().to_vec()
}

fn generate_aes_key(shared_secret: &[u8]) -> Vec<u8> {
    let key = Key::from_slice(shared_secret);
    key.to_vec()
}

fn aes_gcm_encrypt(key: &[u8], plaintext: &[u8], nonce: &[u8], aad: &[u8]) -> Vec<u8> {
    let key = Key::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    cipher.encrypt(nonce, plaintext).expect("encryption failure")
}

fn aes_gcm_decrypt(key: &[u8], ciphertext: &[u8], nonce: &[u8], aad: &[u8]) -> Vec<u8> {
    let key = Key::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    cipher.decrypt(nonce, ciphertext).expect("decryption failure")
}

pub fn encrypt_message(content: &str, recipient: &str) -> Result<String, bigboterror::BigbotError> {
    // Implement message encryption logic here
    Ok(content.to_string())
}

pub fn decrypt_message(content: &str, recipient: &str) -> Result<String, bigboterror::BigbotError> {
    // Implement message decryption logic here
    Ok(content.to_string())
}

pub fn hash_message(content: &str) -> Result<String, bigboterror::BigbotError> {
    // Implement message hashing logic here
    Ok("message_hash".to_string())
}

#[cfg(test)]
mod test {
    use crate::clients::kv::MemoryKVStore;
    use crate::encryption::encryption::EncryptHandler;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_encrypt_handler() {
        let store = Arc::new(MemoryKVStore::default());
        let keyid_store = Arc::new(store.clone());
        let handler = EncryptHandler::new(keyid_store);

        let (uid1, uid2, uid3) = (100, 200, 300);

        let aes_keyid1 = handler.get_or_create_keyid(uid1, "Aes").await.unwrap();
        let aes_keyid11 = handler.get_or_create_keyid(uid1, "Aes").await.unwrap();
        assert_eq!(aes_keyid1, aes_keyid11);

        let aes_keyid2 = handler.get_or_create_keyid(uid2, "Aes").await.unwrap();

        let msg = "This is a message";
        let aad = [b'G', b'E', b'N', b'T', b'O', b'K', b'E', b'N'];
        let encrypted_msg = handler
            .aes_encrypt_message(&aes_keyid1, msg.as_bytes(), aad)
            .await
            .unwrap();
        let ret = handler
            .aes_decrypt_message(&aes_keyid1, encrypted_msg.as_bytes())
            .await;
        assert!(ret.is_ok());
        assert_eq!(ret.unwrap().as_bytes(), msg.as_bytes());
        assert!(handler
            .aes_decrypt_message(&aes_keyid2, encrypted_msg.as_bytes())
            .await
            .is_err());

        let share_12 = handler.negotiate_shared_keyid(uid1, uid2).await.unwrap();
        let share_21 = handler.negotiate_shared_keyid(uid2, uid1).await.unwrap();
        let encrypted_msg = handler
            .aes_encrypt_message(&share_12, msg.as_bytes(), aad)
            .await
            .unwrap();
        let decrypted_msg = handler
            .aes_decrypt_message(&share_21, encrypted_msg.as_bytes())
            .await
            .unwrap();
        assert_eq!(msg.as_bytes(), decrypted_msg.as_bytes());

        let share_13 = handler.negotiate_shared_keyid(uid1, uid3).await.unwrap();
        let r = handler
            .aes_decrypt_message(&share_13, encrypted_msg.as_bytes())
            .await;
        assert!(r.is_err());

        // Test new methods
        let uids = vec![uid1, uid2, uid3];
        let encrypted_messages = handler
            .encrypt_message_for_users(uids.clone(), msg.as_bytes(), aad)
            .await
            .unwrap();
        assert_eq!(encrypted_messages.len(), 3);

        let decrypted_msg = handler
            .decrypt_message_with_shared_key(uid1, uid2, encrypted_messages[0].as_bytes())
            .await
            .unwrap();
        assert_eq!(msg.as_bytes(), decrypted_msg.as_bytes());
    }
}
