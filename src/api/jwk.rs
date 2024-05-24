use crate::clients::postgres::{PGTableKVClient, PG_CLIENT};
use crate::iam::jwt::JWKSEndpoint;

use actix_web::{get, post, put, web, HttpResponse, Responder};
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

lazy_static! {
    pub static ref JWKS_ENDPOINT: JWKSEndpoint = JWKSEndpoint::new(Arc::new(PGTableKVClient::new(
        "jwk".to_string(),
        PG_CLIENT.get().unwrap().clone(),
        "key_id".to_string(),
        "jwk".to_string(),
    )));
}

#[get("/auth/jwks.json")]
async fn jwks() -> impl Responder {
    let resp = match JWKS_ENDPOINT.list().await {
        Ok(jwks) => json!(jwks),
        Err(e) => {
            log::error!("Failed to retrieve JWKs: {}", e);
            json!({"error": format!("{}", e)})
        },
    };
    HttpResponse::Ok().body(resp.to_string())
}

#[get("/auth/jwk/{key_id}")]
async fn get_jwk(path: web::Path<String>) -> impl Responder {
    let key_id = path.into_inner();
    match JWKS_ENDPOINT.get(&key_id).await {
        Ok(Some(jwk)) => HttpResponse::Ok().json(jwk),
        Ok(None) => {
            log::warn!("JWK not found for key_id: {}", key_id);
            HttpResponse::NotFound().json(json!({"error": "JWK not found"}))
        },
        Err(e) => {
            log::error!("Failed to retrieve JWK: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": format!("{}", e)}))
        },
    }
}

#[derive(Deserialize, Clone, Debug)]
struct AddJwkReq {
    key_id: String,
    pem: String,
}

#[post("/auth/jwk")]
async fn add_jwk(info: web::Form<AddJwkReq>) -> impl Responder {
    if info.key_id.is_empty() || info.pem.is_empty() {
        log::warn!("Invalid AddJwkReq: {:?}", info);
        return HttpResponse::BadRequest().body(json!({"error": "key_id and pem fields are required"}).to_string());
    }

    let resp = match JWKS_ENDPOINT
        .add(info.key_id.clone(), info.pem.as_bytes().to_vec())
        .await
    {
        Ok(_) => {
            log::info!("Added JWK with key_id: {}", info.key_id);
            json!({"success": true})
        },
        Err(e) => {
            log::error!("Failed to add JWK: {}", e);
            json!({"error": format!("{}", e)})
        },
    };
    HttpResponse::Ok().body(resp.to_string())
}

#[derive(Deserialize, Clone, Debug)]
struct UpdateJwkReq {
    pem: String,
}

#[put("/auth/jwk/{key_id}")]
async fn update_jwk(path: web::Path<String>, info: web::Form<UpdateJwkReq>) -> impl Responder {
    let key_id = path.into_inner();
    if info.pem.is_empty() {
        log::warn!("Invalid UpdateJwkReq: {:?}", info);
        return HttpResponse::BadRequest().body(json!({"error": "pem field is required"}).to_string());
    }

    let resp = match JWKS_ENDPOINT
        .add(key_id.clone(), info.pem.as_bytes().to_vec())
        .await
    {
        Ok(_) => {
            log::info!("Updated JWK with key_id: {}", key_id);
            json!({"success": true})
        },
        Err(e) => {
            log::error!("Failed to update JWK: {}", e);
            json!({"error": format!("{}", e)})
        },
    };
    HttpResponse::Ok().body(resp.to_string())
}