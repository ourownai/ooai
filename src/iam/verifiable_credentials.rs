use crate::iam::jwt::JWT;
use crate::iam::wallet::Wallet;
use crate::utils::bigboterror::BigbotError;

use jsonwebtoken::{decode, Algorithm, Validation};
use jsonwebtoken::decode_header as jwt_decode_header;use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::str::FromStr;
use web3::types::Address;

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct VerifiableCredential {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    #[serde(rename = "type")]
    pub types: Vec<String>,
    pub id: String,
    pub issuer: String,
    pub issuance_date: String,
    pub proof: Option<Proof>,
    pub credential_subject: CredentialSubject,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CredentialSubject {
    pub id: String,
    pub wallet_address: Address,
    // Add other relevant fields for the credential subject
}

#[derive(Debug)]
pub struct VCBuilder {
    vc: VerifiableCredential,
}

impl VCBuilder {
    pub fn default() -> Self {
        Self {
            vc: VerifiableCredential {
                context: vec![
                    "https://www.w3.org/2018/credentials/v1".to_string(),
                    "https://www.w3.org/2018/credentials/examples/v1".to_string(),
                ],
                types: vec![
                    "VerifiableCredential".to_string(),
                    "RelationshipCredential".to_string(),
                ],
                id: "".to_string(),
                issuer: "".to_string(),
                issuance_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
                proof: None,
                credential_subject: CredentialSubject {
                    id: "".to_string(),
                    wallet_address: Address::default(),
                },
            },
        }
    }

    pub fn add_context(mut self, ctx: String) -> Self {
        self.vc.context.push(ctx);
        self
    }

    pub fn add_type(mut self, t: String) -> Self {
        self.vc.types.push(t);
        self
    }

    pub fn set_id(mut self, id: String) -> Self {
        self.vc.id = id;
        self
    }

    pub fn set_issuer(mut self, issuer: String) -> Self {
        self.vc.issuer = issuer;
        self
    }

    pub fn set_issuance_date(mut self, issuance_date: String) -> Self {
        self.vc.issuance_date = issuance_date;
        self
    }

    pub fn set_proof(mut self, proof: Proof) -> Self {
        self.vc.proof = Some(proof);
        self
    }

    pub fn set_subject_id(mut self, subject_id: String) -> Self {
        self.vc.credential_subject.id = subject_id;
        self
    }

    pub fn set_subject_wallet_address(mut self, wallet_address: Address) -> Self {
        self.vc.credential_subject.wallet_address = wallet_address;
        self
    }

    pub fn build(self) -> VerifiableCredential {
        self.vc
    }
}

impl VerifiableCredential {
    pub fn get_proof(&self) -> Option<&Proof> {
        self.proof.as_ref()
    }

    pub fn get_subject_wallet_address(&self) -> Address {
        self.credential_subject.wallet_address
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Proof {
    #[serde(rename = "type")]
    pub proof_type: String,
    pub created: String,
    pub verification_method: String,
    pub jwt: Option<String>,
}

impl Proof {
    pub fn new(jwt: JWT) -> Self {
        Self {
            proof_type: "JsonWebSignature2020".to_string(),
            created: chrono::Utc::now().to_rfc3339(),
            verification_method: "".to_string(),
            jwt: Some(jwt.to_string()),
        }
    }

    pub fn jwt(&self) -> Option<&String> {
        self.jwt.as_ref()
    }
}

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    pem: String,
}

async fn verify_credential(
    credential: &VerifiableCredential,
    issuer: &str,
    jwks_uri: &str,
) -> Result<(), String> {
    // Extract the JWT from the proof field in the credential
    let jwt = credential
        .proof
        .as_ref()
        .and_then(|proof| proof.jwt.as_ref())
        .ok_or("No JWT found in verifiable credential")?;

    // Download the JWKS from the given URI
    let jwks: Jwks = Client::new()
        .get(jwks_uri)
        .send()
        .await
        .map_err(|e| format!("Failed to download JWKS from {}: {}", jwks_uri, e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse JWKS from {}: {}", jwks_uri, e))?;

    // Find the JWK with a matching key ID in the JWKS
    let header = decode_header(jwt).map_err(|e| format!("Failed to decode JWT header: {}", e))?;
    let jwk = jwks
        .keys
        .iter()
        .find(|key| key.kid == header.kid.unwrap_or_default())
        .ok_or_else(|| {
            format!(
                "No matching key found in JWKS for kid {}",
                header.kid.unwrap_or_default()
            )
        })?;

    // Validate the JWT signature using the JWK
    let validation = Validation::new(Algorithm::RS256);
    let claims = decode::<serde_json::Value>(jwt, jwk.pem.as_ref(), &validation)
        .map_err(|e| format!("JWT validation failed: {}", e))?
        .claims;

    // Check that the issuer claim in the JWT matches the expected issuer
    let issuer_claim = claims
        .get("iss")
        .and_then(|iss| iss.as_str())
        .ok_or("Issuer claim not found in JWT")?;
    if issuer_claim != issuer {
        return Err(format!(
            "Issuer claim does not match expected value: got {}, expected {}",
            issuer_claim, issuer
        ));
    }

    // Check that the audience claim in the JWT is set to "vc-issuer"
    let audience_claim = claims
        .get("aud")
        .and_then(|aud| aud.as_str())
        .ok_or("Audience claim not found in JWT")?;
    if audience_claim != "vc-issuer" {
        return Err(format!(
            "Audience claim does not match expected value: got {}, expected {}",
            audience_claim, "vc-issuer"
        ));
    }

    // All checks passed, the credential is valid
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
enum Degree {
    BachelorOfArts,
    BachelorOfScience,
    MasterOfArts,
    MasterOfScience,
    DoctorOfPhilosophy,
}

fn decode_header(jwt: &str) -> Result<jsonwebtoken::Header, BigbotError> {
    let parts: Vec<_> = jwt.split('.').collect();
    if parts.len() < 2 {
        return Err(BigbotError::RejectedError(
            "Invalid JWT: not enough parts".to_owned(),
        ));
    }
    let header_str = parts[0];
    let header_decoded = base64::engine::general_purpose::STANDARD.decode(header_str).map_err(|e| {
        BigbotError::RejectedError("Invalid JWT: header cannot be decoded".to_string())
    })?;
    let header_json = String::from_utf8_lossy(&header_decoded);
    let header: jsonwebtoken::Header = serde_json::from_str(&header_json)
        .map_err(|e| format!("Failed to parse JWT header: {}", e))?;
    Ok(header)
}

// Define a main function to test the verify_credential function
#[tokio::main]
async fn main() {
    let vc_json = json!({
        "@context": [
            "https://www.w3.org/2018/credentials/v1",
            "https://www.w3.org/2018/credentials/examples/v1"
        ],
        "type": ["VerifiableCredential", "UniversityDegreeCredential"],
        "id": "http://example.edu/credentials/1872",
        "issuer": "https://example.edu/issuers/14",
        "issuanceDate": "2021-01-01T19:23:24Z",
        "credentialSubject": {
            "id": "did:example:ebfeb1f712ebc6f1c276e12ec21",
            "degree": {
                "type": "BachelorDegree",
                "name": "Bachelor of Science and Arts"
            }
        },
        "proof": {
            "type": "JsonWebSignature2020",
            "created": "2021-01-01T19:23:24Z",
            "verificationMethod": "https://example.edu/issuers/14#keys/1",
            "jwt": "eyJhbGciOiJSUzI1NiIsImtpZCI6IjEifQ.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"
        }
    });
    let vc: VerifiableCredential =
        serde_json::from_value(vc_json).expect("Failed to parse verifiable credential JSON");

    // Define the issuer and JWKS URI
    let issuer = "https://example.edu/issuers/14";
    let jwks_uri = "https://example.edu/issuers/14.jwks";

    // Verify the verifiable credential
    match verify_credential(&vc, issuer, jwks_uri).await {
        Ok(_) => println!("Verifiable credential is valid!"),
        Err(e) => println!("Error verifying verifiable credential: {}", e),
    }
}

// Functions for interacting with web3 wallets
pub async fn sign_credential_with_wallet(
    credential: &VerifiableCredential,
    wallet: &Wallet,
) -> Result<String, String> {
    // Sign the credential using the wallet's signing key
    let credential_json = serde_json::to_string(credential).map_err(|e| e.to_string())?;
    let signature = wallet.sign(credential_json.as_bytes());

    // Create a new proof object with the signature
    let proof = Proof {
        proof_type: "JsonWebSignature2020".to_string(),
        created: chrono::Utc::now().to_rfc3339(),
        verification_method: format!("{}#keys-1", wallet.did),
        jwt: Some(base64::engine::general_purpose::STANDARD.encode(&signature)),
    };

    // Add the proof to the credential
    let signed_credential = VerifiableCredential {
        proof: Some(proof),
        ..credential.clone()
    };

    // Return the signed credential as a JSON string
    serde_json::to_string(&signed_credential).map_err(|e| e.to_string())
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
    let signature = base64::engine::general_purpose::STANDARD.decode(&proof.jwt.as_ref().ok_or("No JWT found in the proof")?)
        .map_err(|e| e.to_string())?;

    // Verify the signature using the wallet's verification method
    let credential_json = serde_json::to_string(credential).map_err(|e| e.to_string())?;
    wallet.verify(&signature, credential_json.as_bytes())
}
