use crate::clients::kv::KVStore;
use crate::clients::postgres::PGTableKVClient;
use crate::utils::bigboterror::BigbotError;
use crate::iam::wallet::Wallet;
use crate::iam::did::VerifiableCredential;
use crate::iam::iam::CredentialProof;

use jsonwebtoken::{decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use kafka::producer::AsBytes;
use ockam::compat::rand::{thread_rng, RngCore};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub const JWK_CENTER_URL: &str = "https://yourown.ai/auth/jwks.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct JWT {
    pub(crate) header: jsonwebtoken::Header,
    payload: HashMap<String, String>,
    sign: Option<String>,
}

impl JWT {
    pub fn empty() -> Self {
        Self {
            header: jsonwebtoken::Header::new(Algorithm::RS256),
            payload: HashMap::new(),
            sign: None,
        }
    }

    pub fn add_payload(&mut self, key: String, value: String) {
        self.payload.insert(key, value);
    }

    pub fn get_payloads(&self) -> &HashMap<String, String> {
        &self.payload
    }

    pub fn get_payload(&self, key: &str) -> Option<&String> {
        self.payload.get(key)
    }

    pub(crate) async fn encode(&mut self) -> Result<String, BigbotError> {
        let jwks = JwksCenter::new(JWK_CENTER_URL.to_string());
        let jwk = match jwks.select_random().await? {
            None => return Err(BigbotError::RejectedError(format!("No available jwk"))),
            Some(jwk) => jwk,
        };
        self.header.kid = Some(jwk.kid().to_string());
        self.header.alg = Algorithm::HS256;
        let k = EncodingKey::from_secret(jwk.pem().as_bytes());
        let proof = jsonwebtoken::encode(&self.header, &self.payload, &k).unwrap();
        Ok(proof)
    }

    pub(crate) async fn decode(jwt: &str) -> Result<Self, BigbotError> {
        let err = BigbotError::RejectedError("Invalid verifiable credential".to_string());
        let kid = match decode_header(jwt).map_err(|_x| err.clone())?.kid.as_ref() {
            None => return Err(err),
            Some(kid) => kid.clone(),
        };
        let jwk = match JwksCenter::new(JWK_CENTER_URL.to_string())
            .query(kid.as_str())
            .await?
        {
            None => return Err(err),
            Some(jwk) => jwk.clone(),
        };
        let key = DecodingKey::from_secret(jwk.pem.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims.clear();
        validation.validate_exp = false;
        let token_data = jsonwebtoken::decode::<HashMap<String, String>>(jwt, &key, &validation)
            .map_err(|_e| err.clone())?;
        Ok(Self {
            header: token_data.header,
            payload: token_data.claims,
            sign: None,
        })
    }
}

// Define the function to verify a verifiable credential
#[derive(Debug, Deserialize, Serialize)]
pub struct Jwks {
    jwks: Vec<Jwk>,
}

// we only support hmac algorithm currently
#[derive(Debug, Deserialize, Clone, Serialize)]
struct Jwk {
    kid: String,
    pem: Vec<u8>,
}

impl Jwk {
    pub fn kid(&self) -> &str {
        self.kid.as_str()
    }

    pub fn pem(&self) -> &[u8] {
        self.pem.as_bytes()
    }
}

pub struct JwksCenter {
    url: String,
}

impl JwksCenter {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    async fn select_random(&self) -> Result<Option<Jwk>, BigbotError> {
        let jwks = self.query_from_url().await?;
        if jwks.jwks.len() == 0 {
            return Ok(None);
        }
        let pos = (thread_rng().next_u64() as usize) % jwks.jwks.len();
        Ok(Some(jwks.jwks[pos].clone()))
    }

    async fn query(&self, kid: &str) -> Result<Option<Jwk>, BigbotError> {
        let jwks = self.query_from_url().await?;
        if jwks.jwks.len() == 0 {
            return Ok(None);
        }
        Ok(jwks.jwks.iter().find(|x| &x.kid == kid).map(|x| x.clone()))
    }

    async fn query_from_url(&self) -> Result<Jwks, BigbotError> {
        #[cfg(test)]
        return Ok(Jwks {
            jwks: vec![Jwk {
                kid: "KID".to_string(),
                pem: "pempempempem".as_bytes().to_vec(),
            }],
        });
        let jwks: Jwks = Client::new()
            .get(&self.url)
            .send()
            .await
            .map_err(|e| {
                BigbotError::SystemError(format!(
                    "Failed to download JWKS from {}: {}",
                    self.url, e
                ))
            })?
            .json()
            .await
            .map_err(|e| {
                BigbotError::SystemError(format!("Failed to parse JWKS from {}: {}", self.url, e))
            })?;
        Ok(jwks)
    }
}

#[derive(Clone)]
pub struct JWKSEndpoint {
    pg_client: Arc<PGTableKVClient>,
}

impl JWKSEndpoint {
    pub fn new(pg_client: Arc<PGTableKVClient>) -> Self {
        Self { pg_client }
    }

    pub async fn list(&self) -> Result<Jwks, BigbotError> {
        let jwks: Vec<Jwk> = self
        .pg_client
        .kvs()
        .await
        .map_err(|e| BigbotError::DatabaseError(e.to_string()))?
        .into_iter()
        .map(|(kid, pem)| Jwk {
            kid: String::from_utf8(kid).unwrap(),
            pem,
        })
        .collect();
        Ok(Jwks { jwks: jwks })
    }

    pub async fn get(&self, key_id: &str) -> Result<Option<Jwk>, BigbotError> {
        let jwk = self.pg_client.get(key_id.as_bytes()).await?;
        Ok(jwk.map(|pem| Jwk {
            kid: key_id.to_string(),
            pem,
        }))
    }

    pub async fn add(&self, keyid: String, pem: Vec<u8>) -> Result<(), BigbotError> {
        self.pg_client.set(keyid.as_bytes().to_vec(), pem).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn encode_token(claims: &Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &Header::new(Algorithm::HS256),
        claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
}

pub async fn sign_credential_with_wallet(
    credential: &VerifiableCredential,
    wallet: &Wallet,
) -> Result<VerifiableCredential, String> {
    // Create a JWT from the credential
    let mut jwt = JWT::empty();
    jwt.add_payload("sub".to_string(), credential.subject.clone());
    jwt.add_payload("iss".to_string(), credential.issuer.clone());
    jwt.add_payload("iat".to_string(), credential.issuance_date.clone());

    // Encode the JWT using the wallet's signing key
    let encoded_jwt = jwt.encode().await.map_err(|e| e.to_string())?;

    // Create a new proof object with the encoded JWT as the signature
    let proof = CredentialProof {
        proof_type: "JsonWebSignature2020".to_string(),
        created: chrono::Utc::now().to_rfc3339(),
        verification_method: format!("{}#keys-1", wallet.did),
        signature: encoded_jwt,
    };

    // Add the proof to the credential
    let signed_credential = VerifiableCredential {
        proof: Some(proof),
        ..credential.clone()
    };

    // Return the signed credential
    Ok(signed_credential)
}

pub async fn verify_credential_with_wallet(
    credential: &VerifiableCredential,
    wallet: &Wallet,
) -> Result<bool, String> {
    // Extract the proof from the credential
    let proof = credential
        .proof
        .as_ref()
        .ok_or("No proof found in the credential")?;

    // Extract the signature from the proof
    let signature = &proof.signature;

    // Decode the JWT using the wallet's verification method
    let decoded_jwt = JWT::decode(signature).await.map_err(|e| e.to_string())?;

    // Verify the JWT claims against the credential
    let sub_claim = decoded_jwt
        .get_payload("sub")
        .ok_or("Subject claim not found in JWT")?;
    let iss_claim = decoded_jwt
        .get_payload("iss")
        .ok_or("Issuer claim not found in JWT")?;
    let iat_claim = decoded_jwt
        .get_payload("iat")
        .ok_or("Issued at claim not found in JWT")?;

    if *sub_claim != credential.subject
        || *iss_claim != credential.issuer
        || *iat_claim != credential.issuance_date
    {
        return Ok(false);
    }

    // Verify the signature using the wallet's verification method
    let credential_json = serde_json::to_string(credential).map_err(|e| e.to_string())?;
    let is_valid = wallet.verify(signature.as_bytes(), credential_json.as_bytes());
    Ok(is_valid)
}