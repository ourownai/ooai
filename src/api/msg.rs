use crate::clients::postgres::{PGTableKVClient, PG_CLIENT};
use crate::encryption::encryption::{EncryptHandler, KeysStore};
use crate::messaging::pii_handler::PIIHandler;
use actix_ratelimit::{MemoryStore, MemoryStoreActor, RateLimiter};
use actix_web::middleware::{Logger, NormalizePath};
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use lazy_static::lazy_static;
use log::{error, info};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

lazy_static! {
    pub static ref PII_HANDLER: PIIHandler = {
        let keyid_client = PGTableKVClient::new(
            "user_key".to_string(),
            PG_CLIENT.get().unwrap().clone(),
            "user_id".to_string(),
            "key_id".to_string(),
        );
        let secret_client = PGTableKVClient::new(
            "user_secret".to_string(),
            PG_CLIENT.get().unwrap().clone(),
            "key_id".to_string(),
            "secret".to_string(),
        );
        let vault_store = KeysStore::new(Arc::new(secret_client));
        let encrypt_handler = EncryptHandler::new(vault_store);
        PIIHandler::new(Arc::new(encrypt_handler))
    };
}

#[derive(Deserialize, Clone)]
struct PIIMaskReq {
    msg: String,
    sender_id: i64,
}

#[get("/msg/pii_mask")]
async fn pii_mask(info: web::Query<PIIMaskReq>) -> impl Responder {
    // Input validation
    if info.msg.is_empty() || info.sender_id <= 0 {
        let error_msg = "Invalid input. Message and sender_id are required.";
        error!("{}", error_msg);
        return HttpResponse::BadRequest().json(json!({"error": error_msg}));
    }

    match PII_HANDLER.mask_pii(info.msg.as_str(), info.sender_id).await {
        Ok((masked_msg, token)) => {
            info!("PII masking successful. Token: {}", token);
            HttpResponse::Ok().json(json!({"masked_msg": masked_msg, "token": token}))
        }
        Err(e) => {
            error!("PII masking failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": format!("{}", e)}))
        }
    }
}

#[derive(Deserialize)]
struct ApplyAccessReq {
    token: String,
    sender_id: i64,
    receiver_id: i64,
}

#[get("/msg/apply_access")]
async fn apply_access(info: web::Query<ApplyAccessReq>) -> impl Responder {
    // Input validation
    if info.token.is_empty() || info.sender_id <= 0 || info.receiver_id <= 0 {
        let error_msg = "Invalid input. Token, sender_id, and receiver_id are required.";
        error!("{}", error_msg);
        return HttpResponse::BadRequest().json(json!({"error": error_msg}));
    }

    match PII_HANDLER
        .apply_for_masked_message(info.token.clone(), info.sender_id, info.receiver_id)
        .await
    {
        Ok(r) => {
            info!("Access request successful. VC: {}", r);
            HttpResponse::Ok().json(json!({"vc": r}))
        }
        Err(e) => {
            error!("Access request failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": format!("{}", e)}))
        }
    }
}

#[derive(Deserialize)]
struct PiiUnmaskReq {
    vc: String,
    masked_msg: String,
    sender_id: i64,
    recipient_id: i64,
}

#[get("/msg/pii_unmask")]
async fn pii_unmask(info: web::Query<PiiUnmaskReq>) -> impl Responder {
    // Input validation
    if info.vc.is_empty() || info.masked_msg.is_empty() || info.sender_id <= 0 || info.recipient_id <= 0 {
        let error_msg = "Invalid input. VC, masked_msg, sender_id, and recipient_id are required.";
        error!("{}", error_msg);
        return HttpResponse::BadRequest().json(json!({"error": error_msg}));
    }

    match PII_HANDLER
        .unmask_message(
            info.masked_msg.as_str(),
            info.sender_id,
            info.recipient_id,
            info.vc.clone(),
        )
        .await
    {
        Ok(r) => {
            info!("PII unmasking successful.");
            HttpResponse::Ok().json(json!({"msg": r}))
        }
        Err(e) => {
            error!("PII unmasking failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": format!("{}", e)}))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up logger
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Create a memory store for rate limiting
    let store = MemoryStore::new();

    // Create the Actix Web server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(NormalizePath::default())
            .wrap(
                RateLimiter::new(MemoryStoreActor::from(store.clone()).start())
                    .with_interval(Duration::from_secs(60))
                    .with_max_requests(100),
            )
            .service(pii_mask)
            .service(apply_access)
            .service(pii_unmask)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}