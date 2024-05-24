use async_graphql::{Context, Error, Object, Schema};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::str::FromStr;


use crate::data_exchange::exchange_interfaces::{
    ConnectionInfo, ConnectionType, DataExchangeProcessor,
};
use crate::provider_types::payments::{PaymentProvider, RestPaymentProvider};
use crate::data_streams::grpc::DataExchangeImpl;

#[derive(Deserialize)]
struct ProviderConfig {
    provider_type: String,
    provider_name: String,
    config: HashMap<String, String>,
    metadata: HashMap<String, String>,
}

pub struct RestDataExchangeImpl {
    base_url: String,
    client: reqwest::Client,
}

impl RestDataExchangeImpl {
    pub fn new(base_url: &str) -> Self {
        RestDataExchangeImpl {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl DataExchangeImpl<String, Result<HashMap<String, String>, Box<dyn std::error::Error>>> for RestDataExchangeImpl {
    async fn exchange_data(&self, request: String) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let response = self.client
            .post(&self.base_url)
            .body(request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            let result: HashMap<String, String> = serde_json::from_str(&body)?;
            Ok(result)
        } else {
            Err(format!("REST data exchange failed with status: {}", status).into())
        }
    }
}

pub struct WebhookDataExchangeImpl {
    webhook_base_url: String,
    client: reqwest::Client,
}

impl WebhookDataExchangeImpl {
    pub fn new(webhook_base_url: &str) -> Self {
        WebhookDataExchangeImpl {
            webhook_base_url: webhook_base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl DataExchangeImpl<String, Result<HashMap<String, String>, Box<dyn std::error::Error>>> for WebhookDataExchangeImpl {
    async fn exchange_data(&self, request: String) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let response = self.client
            .post(&self.webhook_base_url)
            .body(request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            let result: HashMap<String, String> = serde_json::from_str(&body)?;
            Ok(result)
        } else {
            Err(format!("Webhook data exchange failed with status: {}", status).into())
        }
    }
}

fn load_provider_configs(directory: &str) -> Result<Vec<ProviderConfig>, String> {
    let mut configs = Vec::new();

    for entry in std::fs::read_dir(directory).map_err(|err| format!("Failed to read directory: {}", err))? {
        let entry = entry.map_err(|err| format!("Failed to read directory entry: {}", err))?;
        let file_path = entry.path();

        if file_path.is_file() {
            let file = File::open(file_path).map_err(|err| format!("Failed to open file: {}", err))?;
            let reader = BufReader::new(file);
            let config: ProviderConfig = serde_json::from_reader(reader).map_err(|err| format!("Failed to parse JSON: {}", err))?;
            configs.push(config);
        }
    }

    Ok(configs)
}

pub struct HasuraContext {
    pub payment_providers: HashMap<String, Arc<dyn PaymentProvider>>,
    pub data_exchange_processor: DataExchangeProcessor,
}

#[async_trait::async_trait]
pub trait HasuraContextFactory {
    async fn create_context(&self) -> HasuraContext;
}

pub struct ArcContextFactory {
    config_directory: String,
    connection_info: ConnectionInfo,
}

#[async_trait::async_trait]
impl HasuraContextFactory for ArcContextFactory {
    async fn create_context(&self) -> HasuraContext {
        let configs = load_provider_configs(&self.config_directory)
            .expect("Failed to load provider configurations");

        let mut payment_providers = HashMap::new();
        let mut data_exchange_processor =
            DataExchangeProcessor::new(self.connection_info.clone());

        for config in configs {
            match config.provider_type.as_str() {
                "payment" => {
                    let provider: Arc<dyn PaymentProvider> = match config.provider_name.as_str() {
                        "rest" => Arc::new(RestPaymentProvider::new(
                            &config.config,
                            &config.metadata,
                        )),
                        _ => panic!("Unsupported payment provider"),
                    };
                    payment_providers.insert(config.provider_name, provider);
                }
                "data_exchange" => match config.provider_name.as_str() {
                    "rest" => {
                        let rest_data_exchange =
                            RestDataExchangeImpl::new(&config.config["base_url"]);
                        data_exchange_processor.register_data_exchange(
                            ConnectionType::Rest,
                            Box::new(rest_data_exchange),
                        );
                    }
                    "webhook" => {
                        let webhook_data_exchange =
                            WebhookDataExchangeImpl::new(&config.config["webhook_base_url"]);
                        data_exchange_processor.register_data_exchange(
                            ConnectionType::Webhook,
                            Box::new(webhook_data_exchange),
                        );
                    }
                    _ => panic!("Unsupported data exchange provider"),
                },
                _ => panic!("Unsupported provider type"),
            }
        }

        HasuraContext {
            payment_providers,
            data_exchange_processor,
        }
    }
}

pub struct HasuraQueryRoot;

#[Object]
impl HasuraQueryRoot {
    async fn payment_provider(
        &self,
        ctx: &Context<'_>,
        provider_name: String,
    ) -> Result<PaymentProviderObject, Error> {
        let context = ctx.data::<HasuraContext>()?;
        let provider = context
            .payment_providers
            .get(&provider_name)
            .ok_or_else(|| Error::new(format!("Payment provider {} not found", provider_name)))?;
        Ok(PaymentProviderObject(provider.clone()))
    }

    async fn data_exchange(
        &self,
        ctx: &Context<'_>,
        connection_type: String,
        request: String,
    ) -> Result<HashMap<String, String>, Error> {
        let context = ctx.data::<HasuraContext>()?;
        let connection_type = ConnectionType::from_str(&connection_type)
            .map_err(|e| Error::new(format!("Invalid connection type: {}", e)))?;
        let result = context
            .data_exchange_processor
            .exchange_data(connection_type, request)
            .await
            .map_err(|e| Error::new(format!("Data exchange failed: {}", e)))?;
        Ok(result)
    }
    
}

pub fn create_schema_with_context(
    config_directory: &str,
    connection_info: ConnectionInfo,
) -> Schema<HasuraQueryRoot, EmptyMutation, EmptySubscription> {
    let context_factory = ArcContextFactory {
        config_directory: config_directory.to_string(),
        connection_info,
    };
    Schema::build(HasuraQueryRoot, EmptyMutation, EmptySubscription)
        .data(context_factory)
        .finish()
}

pub struct EmptyMutation;
pub struct EmptySubscription;

// Define the PaymentProviderObject type
pub struct PaymentProviderObject(Arc<dyn PaymentProvider>);

#[Object]
impl PaymentProviderObject {
    async fn process_payment(
        &self,
        payment_request: PaymentRequest,
    ) -> Result<PaymentResponse, Error> {
        self.0
            .process_payment(
                payment_request.amount,
                &payment_request.currency,
                payment_request.payment_details,
            )
            .map(|transaction_id| PaymentResponse {
                transaction_id,
                status: "success".to_string(),
            })
            .map_err(|e| Error::new(format!("Payment processing failed: {}", e)))
    }
}

// Define the PaymentRequest and PaymentResponse types
pub struct PaymentRequest {
    pub amount: f64,
    pub currency: String,
    pub payment_details: HashMap<String, String>,
}

pub struct PaymentResponse {
    pub transaction_id: String,
    pub status: String,
}