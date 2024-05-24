use async_trait::async_trait;
use cloudevents::Event;
use thiserror::Error;
use rdkafka::config::ClientConfig;
use rdkafka::error::KafkaError;
use futures::SinkExt;
use grpcio::{EnvBuilder, ChannelBuilder};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use rumqttc::QoS;
use rumqttc::v5::{AsyncClient, EventLoop, MqttOptions, ConnectionError};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use std::str::FromStr;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

use crate::data_streams::cloudevents::CloudEventHandler;
use crate::data_streams::grpc::{HelloClientImpl, HelloRequest, HelloClient};
use crate::data_streams::kafka::KafkaSink;
use crate::data_streams::mqtt::{DataExchangeMQTTStream, Error as MqttError};
use crate::utils::bigboterror::BigbotError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionType {
    Grpc,
    Kafka,
    Mqtt,
}

#[derive(Debug)]
pub struct ConnectionInfo {
    pub grpc_address: String,
    pub kafka_bootstrap_servers: String,
    pub mqtt_broker: String,
    pub mqtt_port: u16,
}

#[async_trait]
pub trait DataExchange<Req, Res> {
    async fn call(&self, operator_id: String, _package: String, data: Req) -> Res;
}

#[derive(Error, Debug)]
pub enum DataExchangeError {
    #[error("MQTT error: {0}")]
    MqttError(#[from] MqttError),
    #[error("Kafka error: {0}")]
    KafkaError(#[from] rdkafka::error::KafkaError),
    #[error("gRPC error: {0}")]
    GrpcError(#[from] Box<dyn std::error::Error>),
    #[error("Unknown provider: {0}")]
    UnknownProvider(String),
}

impl FromStr for ConnectionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Grpc" => Ok(ConnectionType::Grpc),
            "Kafka" => Ok(ConnectionType::Kafka),
            "Mqtt" => Ok(ConnectionType::Mqtt),
            _ => Err(format!("Unknown connection type: {}", s)),
        }
    }
}

pub struct DataExchangeProcessor {
    providers: HashMap<String, ConnectionType>,
    connection_info: ConnectionInfo,
}

impl DataExchangeProcessor {
    pub fn new(connection_info: ConnectionInfo) -> Self {
        let config_file = fs::read_to_string("src/agents/providers.json").expect("Unable to read providers.json file");
        let providers_config: HashMap<String, String> =
            serde_json::from_str(&config_file).expect("Unable to parse providers.json file");

        let providers = providers_config
            .into_iter()
            .map(|(name, connection_type)| {
                let connection_type = match connection_type.as_str() {
                    "grpc" => ConnectionType::Grpc,
                    "kafka" => ConnectionType::Kafka,
                    "mqtt" => ConnectionType::Mqtt,
                    _ => panic!("Unsupported provider connection type: {}", connection_type),
                };
                (name, connection_type)
            })
            .collect();

        DataExchangeProcessor {
            providers,
            connection_info,
        }
    }

    pub async fn process_request(
        &self,
        provider_name: &str,
        operator_id: String,
        _package: String,
        request: String,
    ) -> Result<Event, DataExchangeError> {
        match self.providers.get(provider_name) {
            Some(ConnectionType::Grpc) => {
                self.process_grpc_request(operator_id, request).await
            }
            Some(ConnectionType::Kafka) => {
                self.process_kafka_request(operator_id, request).await
            }
            Some(ConnectionType::Mqtt) => {
                self.process_mqtt_request(operator_id, request).await
            }
            None => Err(DataExchangeError::UnknownProvider(provider_name.to_string())),
        }
    }

    async fn process_grpc_request(
        &self,
        operator_id: String,
        request: String,
    ) -> Result<Event, DataExchangeError> {
        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(&self.connection_info.grpc_address);
        let client = HelloClientImpl::new(ch);
        let mut req = HelloRequest::new();
        req.set_name(request);
        let response = client.say_hello(&req).await.map_err(DataExchangeError::GrpcError)?;
        let event = CloudEventHandler::create_cloudevent(operator_id, response.message);
        Ok(event)
    }    

    async fn process_kafka_request(
        &self,
        operator_id: String,
        request: String,
    ) -> Result<Event, DataExchangeError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &self.connection_info.kafka_bootstrap_servers)
            .create()
            .map_err(DataExchangeError::KafkaError)?;
        let mut kafka_sink = KafkaSink::new(producer, "package".to_string());
        let event = CloudEventHandler::create_cloudevent(operator_id, request);
        kafka_sink
            .send(event)
            .await
            .map_err(DataExchangeError::KafkaError)?;
        Ok(event)
    }

    async fn process_mqtt_request(
        &self,
        operator_id: String,
        request: String,
    ) -> Result<Event, DataExchangeError> {
        let (mqtt_client, mqtt_eventloop) = create_mqtt_client(
            &self.connection_info.mqtt_broker,
            self.connection_info.mqtt_port,
        )
        .await;
        let mut mqtt_stream = DataExchangeMQTTStream::new(mqtt_client, mqtt_eventloop);
        mqtt_stream
            .publish("package", request.as_bytes(), QoS::AtLeastOnce)
            .await
            .map_err(DataExchangeError::MqttError)?;
        let envelope = mqtt_stream
            .next()
            .await
            .ok_or_else(|| DataExchangeError::MqttError(MqttError::ConnectionError(ConnectionError::Disconnected)))?
            .map_err(DataExchangeError::MqttError)?;
        Ok(envelope.data)
    }

    pub async fn exchange_data(
        &self,
        connection_type: ConnectionType,
        _request: String,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        match connection_type {
            ConnectionType::Grpc => {
                // Handle gRPC exchange
                todo!()
            }
            ConnectionType::Kafka => {
                // Handle Kafka exchange
                todo!()
            }
            ConnectionType::Mqtt => {
                // Handle MQTT exchange
                todo!()
            }
        }
    }
}

impl futures::Sink<Event> for KafkaSink {
    type Error = KafkaError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let producer = self.producer.as_mut().unwrap();
        match producer.poll(Timeout::After(Duration::from_millis(0))) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    fn start_send(self: Pin<&mut Self>, item: Event) -> Result<(), Self::Error> {
        let record = FutureRecord::to(&self.topic)
            .payload(&serde_json::to_string(&item).unwrap())
            .key(&item.id().to_string());

        let producer = self.producer.as_mut().unwrap();
        producer.send(record, Timeout::Never).map(|_| ()).map_err(|(e, _)| e)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let producer = self.producer.as_mut().unwrap();
        match producer.flush(Timeout::After(Duration::from_millis(0))) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }
}

async fn create_mqtt_client(broker: &str, port: u16) -> (AsyncClient, EventLoop) {
    let mut options = MqttOptions::new("test-client", broker, port);
    options.set_keep_alive(Duration::from_secs(5));

    let (client, eventloop) = AsyncClient::new(options, 10);
    (client, eventloop)
}
