use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use web3::types::Address;

use crate::iam::jwt::{sign_credential_with_wallet, verify_credential_with_wallet};
use crate::iam::keycloak_provider::UserRepresentation;
use crate::iam::wallet::{Wallet, WalletAddress};
use crate::iam::did::VerifiableCredential;
use crate::utils::bigboterror::BigbotError;

#[derive(Debug, Serialize, Deserialize)]
struct Token {
    access_token: String,
    expires_in: u64,
    refresh_expires_in: u64,
    refresh_token: String,
    token_type: String,
    session_state: String,
    scope: String,
    #[serde(default)]
    created_at: u64,
}

#[derive(Debug, Clone)]
struct KeycloakAdmin {
    client: Client,
    base_url: String,
    realm_name: String,
    client_id: String,
    client_secret: String,
    admin_username: String,
    admin_password: String,
}

impl KeycloakAdmin {
    fn new(
        base_url: &str,
        realm_name: &str,
        client_id: &str,
        client_secret: &str,
        admin_username: &str,
        admin_password: &str,
    ) -> Self {
        KeycloakAdmin {
            client: Client::new(),
            base_url: base_url.to_string(),
            realm_name: realm_name.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            admin_username: admin_username.to_string(),
            admin_password: admin_password.to_string(),
        }
    }

    async fn get_users(&self) -> Result<Vec<KeycloakUserModel>, BigbotError> {
        let url = format!("{}/admin/realms/{}/users", self.base_url, self.realm_name);
        let response = self.client.get(&url).send().await.map_err(|e| BigbotError::UserGetError(e.to_string()))?;
        let users: Vec<KeycloakUserModel> = response.json().await.map_err(|e| BigbotError::UserGetError(e.to_string()))?;
        Ok(users)
    }

    async fn create_user(&self, user: UserRepresentation) -> Result<KeycloakUserModel, reqwest::Error> {
        let url = format!("{}/admin/realms/{}/users", self.base_url, self.realm_name);
        let response = self.client.post(&url).json(&user).send().await?;
        let created_user: KeycloakUserModel = response.json().await?;
        Ok(created_user)
    }

    async fn update_user(&self, user: UserRepresentation) -> Result<KeycloakUserModel, reqwest::Error> {
        let url = format!("{}/admin/realms/{}/users/{}", self.base_url, self.realm_name, user.id);
        let response = self.client.put(&url).json(&user).send().await?;
        let updated_user: KeycloakUserModel = response.json().await?;
        Ok(updated_user)
    }

    async fn delete_user(&self, username: &str) -> Result<bool, reqwest::Error> {
        let url = format!("{}/admin/realms/{}/users/{}", self.base_url, self.realm_name, username);
        let response = self.client.delete(&url).send().await?;
        Ok(response.status().is_success())
    }

    async fn get_user(&self, user_id: &str) -> Result<UserRepresentation, BigbotError> {
        let url = format!("{}/admin/realms/{}/users/{}", self.base_url, self.realm_name, user_id);
        let response = self.client.get(&url).send().await.map_err(|e| BigbotError::UserGetError(e.to_string()))?;
        let user: UserRepresentation = response.json().await.map_err(|e| BigbotError::UserGetError(e.to_string()))?;
        Ok(user)
    }
}

struct KeycloakController {
    keycloak_admin: KeycloakAdmin,
}

impl KeycloakController {
    fn new(
        base_url: &str,
        realm_name: &str,
        client_id: &str,
        client_secret: &str,
        admin_username: &str,
        admin_password: &str,
    ) -> Self {
        let keycloak_admin = KeycloakAdmin::new(
            base_url,
            realm_name,
            client_id,
            client_secret,
            admin_username,
            admin_password,
        );
        KeycloakController { keycloak_admin }
    }

    async fn issue_credential(
        &self,
        user_id: &str,
        credential: VerifiableCredential,
    ) -> Result<VerifiableCredential, BigbotError> {
        let wallet = self.get_user_wallet(user_id).await?;
    
        let signed_credential = sign_credential_with_wallet(&credential, &wallet)
            .await
            .map_err(BigbotError::CredentialSignError)?;
    
        // Store the signed credential in Keycloak or a credential repository
    
        Ok(signed_credential)
    }

    async fn verify_credential(
        &self,
        credential: VerifiableCredential,
    ) -> Result<bool, BigbotError> {
        let issuer_wallet = self.get_user_wallet(&credential.issuer).await?;
    
        let is_valid = verify_credential_with_wallet(&credential, &issuer_wallet)
            .await
            .map_err(BigbotError::CredentialVerificationError)?;
    
        Ok(is_valid)
    }

    async fn create_user_wallet(&self, user_id: &str) -> Result<Wallet, BigbotError> {
        let wallet = Wallet::new_wallet().await;

        let mut user_representation = self.keycloak_admin.get_user(user_id).await.map_err(|e| BigbotError::UserGetError(e.to_string()))?;
        user_representation.attributes.insert("wallet_id".to_string(), vec![wallet.id.clone()]);
        user_representation.attributes.insert("wallet_public_key".to_string(), vec![wallet.public_key.clone()]);
        self.keycloak_admin.update_user(user_representation).await.map_err(|e| BigbotError::UserUpdateError(e.to_string()))?;

        Ok(wallet)
    }

    async fn get_user_wallet(&self, user_id: &str) -> Result<Wallet, BigbotError> {
        let user_representation = self.keycloak_admin.get_user(user_id).await?;
        let wallet_id = user_representation.attributes.get("wallet_id").and_then(|v| v.first().cloned());
        let wallet_public_key = user_representation.attributes.get("wallet_public_key").and_then(|v| v.first().cloned());
    
        match (wallet_id, wallet_public_key) {
            (Some(id), Some(public_key)) => Ok(Wallet {
                id,
                public_key,
                did: "".to_string(),
                identity_doc: "".to_string(),
                credentials: HashMap::new(),
                keys: vec![],
                addresses: vec![],
                preferred_address: WalletAddress(Address::default()),
                base_currency: "".to_string(),
                payment_thresholds: HashMap::new(),
            }),
            _ => Err(BigbotError::WalletNotFound),
        }
    }

    async fn openid_token(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Token, BigbotError> {
        let url = format!("{}/realms/{}/protocol/openid-connect/token", self.keycloak_admin.base_url, self.keycloak_admin.realm_name);
        let params = [
            ("grant_type", "password"),
            ("client_id", &self.keycloak_admin.client_id),
            ("client_secret", &self.keycloak_admin.client_secret),
            ("username", username),
            ("password", password),
        ];
        let response = self.keycloak_admin.client.post(&url).form(&params).send().await.map_err(BigbotError::OpenIDTokenError)?;
        let token: Token = response.json().await.map_err(BigbotError::OpenIDTokenError)?;
        Ok(token)
    }

    async fn authenticate(&self, token: &Token) -> Result<KeycloakUserModel, BigbotError> {
        let url = format!("{}/realms/{}/protocol/openid-connect/userinfo", self.keycloak_admin.base_url, self.keycloak_admin.realm_name);
        let response = self.keycloak_admin.client.get(&url)
            .bearer_auth(&token.access_token)
            .send()
            .await
            .map_err(|_| BigbotError::AuthenticationError("Failed to authenticate user".to_string()))?;
        let user: KeycloakUserModel = response.json().await.map_err(|_| BigbotError::AuthenticationError("Failed to parse user info".to_string()))?;
        Ok(user)
    }

    async fn logout(&self, token: &Token) -> Result<bool, BigbotError> {
        let url = format!("{}/realms/{}/protocol/openid-connect/logout", self.keycloak_admin.base_url, self.keycloak_admin.realm_name);
        let params = [
            ("client_id", &self.keycloak_admin.client_id),
            ("client_secret", &self.keycloak_admin.client_secret),
            ("refresh_token", &token.refresh_token),
        ];
        let response = self.keycloak_admin.client.post(&url).form(&params).send().await.map_err(BigbotError::LogoutError)?;
        Ok(response.status().is_success())
    }
}

#[tokio::main]
async fn main() -> Result<(), BigbotError> {
    // ... (existing code)
    Ok(())
}

struct KeycloakUserManager {
    keycloak_admin: KeycloakAdmin,
}

impl KeycloakUserManager {
    fn new(keycloak_admin: &KeycloakAdmin) -> Self {
        KeycloakUserManager {
            keycloak_admin: keycloak_admin.clone(),
        }
    }

    async fn filter(
        &self,
        field: &str,
        value: &str,
    ) -> Result<Option<KeycloakUserModel>, BigbotError> {
        let users = self
            .keycloak_admin
            .get_users()
            .await
            .map_err(|e| BigbotError::UserFilterError(e.to_string()))?;

        let filtered_user = users.into_iter().find(|user| match field {
            "username" => user.username == Some(value.to_string()),
            "email" => user.email == Some(value.to_string()),
            _ => false,
        });

        Ok(filtered_user)
    }

    async fn create_user(
        &self,
        user: &KeycloakUserModel,
    ) -> Result<KeycloakUserModel, BigbotError> {
        let user_representation = user.to_user_representation();
        let created_user = self
            .keycloak_admin
            .create_user(user_representation)
            .await
            .map_err(|e| BigbotError::UserCreateError(e.to_string()))?;
        Ok(created_user)
    }

    async fn update_user(
        &self,
        user: &KeycloakUserModel,
    ) -> Result<KeycloakUserModel, BigbotError> {
        let user_representation = user.to_user_representation();
        let updated_user = self
            .keycloak_admin
            .update_user(user_representation)
            .await
            .map_err(|e| BigbotError::UserUpdateError(e.to_string()))?;
        Ok(updated_user)
    }

    async fn delete_user(&self, username: &str) -> Result<bool, BigbotError> {
        let deleted = self
            .keycloak_admin
            .delete_user(username)
            .await
            .map_err(|e| BigbotError::UserDeleteError(e.to_string()))?;
        Ok(deleted)
    }

    async fn create_user_wallet(&self, user_id: &str) -> Result<Wallet, BigbotError> {
        let wallet = Wallet::new_wallet().await;

        let mut user_representation = self.keycloak_admin.get_user(user_id).await.map_err(|e| BigbotError::UserGetError(e.to_string()))?;
        user_representation
            .attributes
            .insert("wallet_id".to_string(), vec![wallet.id.clone()]);
        user_representation
            .attributes
            .insert("wallet_public_key".to_string(), vec![wallet.public_key.clone()]);
        self.keycloak_admin.update_user(user_representation).await.map_err(|e| BigbotError::UserUpdateError(e.to_string()))?;

        Ok(wallet)
    }

    async fn get_user_wallet(&self, user_id: &str) -> Result<Wallet, BigbotError> {
        let user_representation = self.keycloak_admin.get_user(user_id).await.map_err(|e| BigbotError::UserGetError(e.to_string()))?;
        let wallet_id = user_representation
            .attributes
            .get("wallet_id")
            .and_then(|v| v.first().cloned());
        let wallet_public_key = user_representation
            .attributes
            .get("wallet_public_key")
            .and_then(|v| v.first().cloned());
    
        match (wallet_id, wallet_public_key) {
            (Some(id), Some(public_key)) => Ok(Wallet {
                id,
                public_key,
                did: "".to_string(),
                identity_doc: "".to_string(),
                credentials: HashMap::new(),
                keys: vec![],
                addresses: vec![],
                preferred_address: WalletAddress(Address::default()),
                base_currency: "".to_string(),
                payment_thresholds: HashMap::new(),
            }),
            _ => Err(BigbotError::WalletNotFound),
        }
    }

    async fn issue_credential(
        &self,
        user_id: &str,
        credential: VerifiableCredential,
    ) -> Result<VerifiableCredential, BigbotError> {
        let wallet = self.get_user_wallet(user_id).await?;

        let signed_credential = sign_credential_with_wallet(&credential, &wallet)
            .await
            .map_err(BigbotError::CredentialSignError)?;

        // Store the signed credential in Keycloak or a credential repository
        // ...

        Ok(signed_credential)
    }

    async fn verify_credential(
        &self,
        credential: VerifiableCredential,
    ) -> Result<bool, BigbotError> {
        let issuer_wallet = self.get_user_wallet(&credential.issuer).await?;

        let is_valid = verify_credential_with_wallet(&credential, &issuer_wallet)
            .await
            .map_err(|e| BigbotError::CredentialVerificationError(e.to_string()))?;

        Ok(is_valid)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct KeycloakUserModel {
    id: Option<String>,
    username: Option<String>,
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    enabled: Option<bool>,
    email_verified: Option<bool>,
    attributes: HashMap<String, Vec<String>>,
}


impl KeycloakUserModel {
    fn to_user_representation(&self) -> UserRepresentation {
        UserRepresentation {
            id: self.id.clone(),
            username: self.username.clone(),
            email: self.email.clone(),
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            enabled: self.enabled,
            email_verified: self.email_verified,
            attributes: self.attributes.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialProof {
    pub proof_type: String,
    pub created: String,
    pub verification_method: String,
    pub signature: String,
}

impl VerifiableCredential {
    fn new(
        issuer: String,
        subject: String,
        credential_type: Vec<String>,
        credential_subject: serde_json::Value,
    ) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let issuance_date = chrono::Utc::now().to_rfc3339();
        VerifiableCredential {
            id,
            issuer,
            subject,
            issuance_date,
            expiration_date: None,
            credential_type,
            credential_subject,
            proof: None,
            context: vec!["https://www.w3.org/2018/credentials/v1".to_string()],
            types: vec!["VerifiableCredential".to_string()],
        }
    }

    fn set_expiration_date(&mut self, expiration_date: String) {
        self.expiration_date = Some(expiration_date);
    }

    fn set_proof(&mut self, proof: CredentialProof) {
        self.proof = Some(proof);
    }
}

impl CredentialProof {
    fn new(
        proof_type: String,
        verification_method: String,
        signature: String,
    ) -> Self {
        let created = chrono::Utc::now().to_rfc3339();
        CredentialProof {
            proof_type,
            created,
            verification_method,
            signature,
        }
    }
}