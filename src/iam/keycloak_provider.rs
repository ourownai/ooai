// keycloak_provider.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::iam::jwt::{Claims, encode_token, decode_token};
use crate::utils::random::generate_random_alphanumeric_string;

#[derive(Debug, Serialize, Deserialize)]
pub struct RealmCreationRequest {
    pub id: String,
    pub realm: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCreationRequest {
    pub username: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRepresentation {
    pub id: String,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub enabled: bool,
    pub email_verified: bool,
    pub attributes: HashMap<String, Vec<String>>,
}

pub async fn create_realm(client: &Client, admin_token: &str, realm_name: &str) -> Result<(), reqwest::Error> {
    let create_realm_url = "http://keycloak-server/admin/realms";
    let realm_request = RealmCreationRequest {
        id: realm_name.to_string(),
        realm: realm_name.to_string(),
    };

    client.post(create_realm_url)
        .bearer_auth(admin_token)
        .json(&realm_request)
        .send()
        .await?;

    Ok(())
}

pub async fn create_user(client: &Client, admin_token: &str, realm_name: &str, username: &str) -> Result<UserRepresentation, reqwest::Error> {
    let create_user_url = format!("http://keycloak-server/admin/realms/{}/users", realm_name);
    let user_request = UserCreationRequest {
        username: username.to_string(),
        enabled: true,
    };

    let response = client.post(&create_user_url)
        .bearer_auth(admin_token)
        .json(&user_request)
        .send()
        .await?;

    response.json::<UserRepresentation>().await
}
