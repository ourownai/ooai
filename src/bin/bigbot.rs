use actix_web::{web, App, HttpServer, middleware::Logger};
use api::jwk::{add_jwk, jwks};
use api::msg::{apply_access, pii_mask, pii_unmask};
use clients::postgres::init_postgres_client_from_env;
use config::ServerConfig;
use error::BigbotError;
use routes::configure_routes;
use logger::setup_logger;

#[actix_web::main]
async fn main() -> Result<(), BigbotError> {
    // Initialize PostgreSQL client
    init_postgres_client_from_env().await?;

    // Load server configuration
    let config = ServerConfig::from_env();

    // Set up logger
    setup_logger()?;

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(configure_routes)
            .app_data(web::Data::new(config.clone()))
    })
    .bind(config.server_addr())?
    .run()
    .await?;

    Ok(())
}

// src/config.rs
#[derive(Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| "9090".to_string())
            .parse()
            .unwrap_or(9090);
        ServerConfig { host, port }
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// src/routes.rs
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(pii_mask)
            .service(pii_unmask)
            .service(apply_access)
            .service(jwks)
            .service(add_jwk),
    );
}

// src/error.rs
#[derive(Debug)]
pub enum BigbotError {
    // Define custom error variants
    // Example:
    // DatabaseError(String),
    // ConfigurationError(String),
    // ...
}

impl std::fmt::Display for BigbotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Implement error display
        // Example:
        // match self {
        //     BigbotError::DatabaseError(err) => write!(f, "Database error: {}", err),
        //     BigbotError::ConfigurationError(err) => write!(f, "Configuration error: {}", err),
        //     ...
        // }
        write!(f, "BigbotError")
    }
}

impl actix_web::ResponseError for BigbotError {
    // Implement actix_web::ResponseError for BigbotError
    // Example:
    // fn error_response(&self) -> actix_web::HttpResponse {
    //     match self {
    //         BigbotError::DatabaseError(_) => actix_web::HttpResponse::InternalServerError().finish(),
    //         BigbotError::ConfigurationError(_) => actix_web::HttpResponse::BadRequest().finish(),
    //         ...
    //     }
    // }
}

// src/logger.rs
pub fn setup_logger() -> Result<(), BigbotError> {
    // Set up logger configuration
    // Example using env_logger:
    env_logger::init();
    Ok(())
}
