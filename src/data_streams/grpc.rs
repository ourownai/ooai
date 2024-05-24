use async_trait::async_trait;
use grpcio::Channel;
use std::collections::HashMap;

use crate::protos::my_grpc_service::{MyServiceClient, MyRequest};

// Define request and response types for the gRPC service
pub struct HelloRequest {
    name: String,
}

impl HelloRequest {
    pub fn new() -> Self {
        HelloRequest {
            name: String::new(),
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

pub struct HelloResponse {
    pub message: String,
}

// Define the gRPC client trait
#[async_trait]
pub trait HelloClient {
    async fn say_hello(&self, req: &HelloRequest) -> Result<HelloResponse, Box<dyn std::error::Error>>;
}

pub struct HelloClientImpl {
    client: MyServiceClient,
}

impl HelloClientImpl {
    pub fn new(channel: Channel) -> Self {
        let client = MyServiceClient::new(channel);
        HelloClientImpl { client }
    }
}

impl HelloClient for HelloClientImpl {
    async fn say_hello(&self, req: &HelloRequest) -> Result<HelloResponse, Box<dyn std::error::Error>> {
        let request = MyRequest {
            data: req.name.clone(),
        };
        let response = self.client.my_method(&request).await?;
        Ok(HelloResponse {
            message: response.result,
        })
    }
}

// Define the connection information struct
pub struct ConnectionInfo {
    pub grpc_address: String,
}

// Define the data exchange trait
#[async_trait]
pub trait DataExchange<T, R> {
    async fn call(
        &self,
        operator_id: String,
        package: String,
        data: T,
    ) -> R;
}

// Define the data exchange implementation trait
#[async_trait]
pub trait DataExchangeImpl<T, R> {
    async fn exchange_data(&self, request: T) -> R;
}

// Implement the gRPC data exchange
pub struct GrpcDataExchangeImpl<C>
where
    C: HelloClient + Send + Sync,
{
    client: C,
}

impl<C> GrpcDataExchangeImpl<C>
where
    C: HelloClient + Send + Sync,
{
    pub fn new(connection_info: Option<&ConnectionInfo>, client_factory: impl Fn(String) -> C) -> Self {
        let address = connection_info.map(|info| &info.grpc_address).unwrap_or("localhost:50051");
        let client = client_factory(address.to_string());
        GrpcDataExchangeImpl { client }
    }

    pub fn establish_connection(&mut self, address: &str, client_factory: impl Fn(String) -> C) {
        let client = client_factory(address.to_string());
        self.client = client;
    }
}

#[async_trait]
impl<C> DataExchange<String, Result<HashMap<String, String>, Box<dyn std::error::Error>>> for GrpcDataExchangeImpl<C>
where
    C: HelloClient + Send + Sync,
{
    async fn call(
        &self,
        operator_id: String,
        package: String,
        data: String,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let mut req = HelloRequest::new();
        req.set_name(data);
        let response = self.client.say_hello(&req).await?;
        let mut result = HashMap::new();
        result.insert("response".to_string(), response.message);
        Ok(result)
    }
}

#[async_trait]
impl<C> DataExchangeImpl<String, Result<HashMap<String, String>, Box<dyn std::error::Error>>> for GrpcDataExchangeImpl<C>
where
    C: HelloClient + Send + Sync,
{
    async fn exchange_data(&self, request: String) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let mut req = HelloRequest::new();
        req.set_name(request);
        let response = self.client.say_hello(&req).await?;
        let mut result = HashMap::new();
        result.insert("response".to_string(), response.message);
        Ok(result)
    }
}
