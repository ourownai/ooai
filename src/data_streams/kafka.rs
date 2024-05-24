//! This module provides functionality for consuming and producing messages using Kafka.
//!
//! `KafkaSink` is a wrapper of `FutureProducer` and implements the `Sink` trait for consuming items
//! and sending them to a Kafka topic. It provides an asynchronous `consume` method that serializes
//! the input item and sends it as a message to the specified Kafka topic.
//!
//! `KafkaStream` encapsulates the streaming of messages from a Kafka topic, converting them into
//! CloudEvents and wrapping them in an `Envelope`. It leverages `rdkafka`'s `MessageStream` for
//! consuming messages and utilizes the CloudEvents SDK for message conversion.
//!
//! Fields:
//! - `message_stream`: A `MessageStream` from `rdkafka` representing the stream of messages from Kafka.
//! - `tx`: An `UnboundedSender` for sending `Reply<BorrowedMessage<'a>>` back to the sender, used for
//!   acknowledging message processing outcomes.
//!
//! `KafkaStream` implements the `Stream` trait, yielding `Result<Envelope<Event, BorrowedMessage<'a>>, KafkaConsumerError>`
//! where each item represents either a successfully enveloped CloudEvent or an error indicating a failure in
//! message consumption or conversion.
//!
//! `KafkaConsumerError` is an enum representing the types of errors that can occur during message consumption
//! and conversion to CloudEvents, including `KafkaError` for errors from the Kafka consumer and `CloudEventError`
//! for errors during the conversion to CloudEvents.
//!
//! `DataExchangeKafkaConsumer` provides functionality to consume messages from a Kafka topic using `StreamConsumer`
//! and process them asynchronously. It facilitates message commitment and error handling through a spawned Tokio
//! task, which listens for replies indicating the success or failure of message processing.
//!
//! Usage:
//! - `KafkaSink::new` creates a new instance of `KafkaSink` with a specified `FutureProducer` and topic.
//! - `KafkaStream::new` initializes a new instance of `KafkaStream` with a specified message stream and sender for replies.
//! - `DataExchangeKafkaConsumer::new` creates a new Kafka consumer for data exchange.
//! - `DataExchangeKafkaConsumer::stream` returns a stream of enveloped events or errors, integrating Kafka consumption
//!   with application-specific processing logic and asynchronous acknowledgment of message processing results.
//! 

use async_trait::async_trait;
use cloudevents::binding::rdkafka::MessageExt;
use cloudevents::Event;
use futures_core::Stream;
use pin_project_lite::pin_project;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, MessageStream, StreamConsumer};
use rdkafka::message::{BorrowedMessage, OwnedHeaders, OwnedMessage};
use rdkafka::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::Serialize;
use serde_json::to_value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tracing::{error, warn};

use crate::utils::bigboterror::BigBotError;
use crate::data_exchange::exchange_interfaces::DataExchange;
use crate::data_streams;
use crate::data_exchange::exchange_core::{Envelope, Sink, Reply};
use rdkafka::message::Headers;

pub struct KafkaDataExchangeImpl {
    producer: FutureProducer,
    consumer: StreamConsumer,
}

impl KafkaDataExchangeImpl {
    pub fn new(producer_config: &str, consumer_config: &str, topic: &str, group_id: &str) -> Result<KafkaDataExchangeImpl, Box<dyn Error>> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", producer_config)
            .create()?;

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", consumer_config)
            .set("group.id", group_id)
            .set("auto.offset.reset", "earliest")
            .create()?;

        consumer.subscribe(&[topic])?;

        Ok(KafkaDataExchangeImpl { producer, consumer })
    }

pub async fn send_message_with_headers(&self, topic: &str, message: &str, headers: &HashMap<String, String>) -> Result<(), BigbotError> {
    let mut record = FutureRecord::to(topic).key("").payload(message);
    let mut owned_headers = OwnedHeaders::new();
    for (key, value) in headers {
        owned_headers.insert(key, value);
    }
    record = record.headers(owned_headers);
    self.producer.send(record, Duration::from_secs(0)).await.map_err(BigbotError::from)?;
    Ok(())
}

    pub async fn receive_message_with_headers(&self, topic: &str) -> Result<(String, HashMap<String, String>), Box<dyn Error>> {
        loop {
            match self.consumer.recv().await {
                Ok(message) => {
                    if message.topic() == topic {
                        if let Some(payload) = message.payload() {
                            let message_str = std::str::from_utf8(payload)?;
                            let headers_map = message
                                .headers()
                                .map(|h| {
                                    h.iter()
                                        .map(|(key, value)| {
                                            (
                                                key.to_string(),
                                                std::str::from_utf8(value).unwrap().to_string(),
                                            )
                                        })
                                        .collect()
                                })
                                .unwrap_or_default();
                            return Ok((message_str.to_string(), headers_map));
                        }
                    }
                }
                Err(e) => {
                    println!("Error while polling for messages: {:?}", e);
                }
            }
        }
    }
}

#[async_trait]
impl DataExchange<String, Result<String, Box<dyn std::error::Error + Send + Sync>>> for KafkaDataExchangeImpl {
    async fn call(
        &self,
        operator_id: String,
        package: String,
        data: String,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let topic = data_streams::topics::dynamic_topic(data_streams::topics::INFERENCE_TOPIC_PREFIX, &operator_id);
        let metadata = HashMap::new(); // Modify this if you have specific metadata to send
        self.send_message_with_headers(&topic, &data, &metadata).await?;
        let (result, _) = self.receive_message_with_headers(&topic).await?;
        Ok(result)
    }
}

pub struct KafkaSink {
    pub producer: FutureProducer,
    pub topic: String,
}

impl KafkaSink {
    pub fn new(producer: FutureProducer, topic: String) -> Self {
        Self { producer, topic }
    }
}

#[derive(Debug)]
pub struct KafkaProducerError {
    kafka_error: rdkafka::error::KafkaError,
    message: OwnedMessage,
}

impl Display for KafkaProducerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "KafkaProducerError: {:?}, Message: {:?}", self.kafka_error, self.message)
    }
}

impl From<KafkaProducerError> for BigbotError {
    fn from(err: KafkaProducerError) -> Self {
        BigbotError::KafkaError(format!("KafkaProducerError: {:?}, Message: {:?}", err.kafka_error, err.message))
    }
}

#[derive(Debug)]
pub enum SinkError {
    CodecError(serde_json::Error),
    InternalError(Box<dyn Error + Send + Sync>),
    Cancelled,
    KafkaProducerError(KafkaProducerError),
}

impl Display for SinkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SinkError::CodecError(e) => write!(f, "CodecError: {}", e),
            SinkError::InternalError(e) => write!(f, "InternalError: {}", e),
            SinkError::Cancelled => write!(f, "Cancelled"),
            SinkError::KafkaProducerError(e) => write!(f, "{}", e),
        }
    }
}

impl Error for SinkError {}

#[async_trait]
impl<'a, T> Sink<T> for KafkaSink
where
    T: 'a + Serialize + Send,
{
    type Error = SinkError;

    async fn consume(&self, item: T) -> Result<(), SinkError>
    where
        T: 'async_trait,
    {
        let msg = to_value(item).map_err(|e| SinkError::CodecError(e))?;
        let msg_str = msg.to_string();
        let record = FutureRecord::to(self.topic.as_str())
            .payload(msg_str.as_bytes())
            .key("fake");
        let delivery_fut = self
            .producer
            .send_result(record)
            .map_err(|(e, r)| SinkError::KafkaProducerError(KafkaProducerError { kafka_error: e, message: r }))?;
        let (partition, offset) = delivery_fut
            .await
            .map_err(|_canceled| SinkError::Cancelled)?
            .map_err(|(e, _msg)| SinkError::InternalError(Box::new(e)))?;
        println!(
            "Sent message to partition {} and offset {}",
            partition, offset
        );
        Ok(())
    }
}

pin_project! {
    pub struct KafkaStream<'a> {
        #[pin]
        message_stream: MessageStream<'a>,
        tx: UnboundedSender<Reply<BorrowedMessage<'a>>>,
    }
}

impl<'a> KafkaStream<'a> {
    pub fn new(message_stream: MessageStream<'a>, tx: UnboundedSender<Reply<BorrowedMessage<'a>>>) -> Self {
        Self { message_stream, tx }
    }
}

#[derive(Debug)]
pub enum KafkaConsumerError {
    KafkaError(rdkafka::error::KafkaError),
    CloudEventError(cloudevents::message::Error),
}

impl Display for KafkaConsumerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KafkaConsumerError::KafkaError(e) => write!(f, "KafkaError: {}", e),
            KafkaConsumerError::CloudEventError(e) => write!(f, "CloudEventError: {}", e),
        }
    }
}

impl Error for KafkaConsumerError {}

impl<'a> Stream for KafkaStream<'a> {
    type Item = Result<Envelope<Event, BorrowedMessage<'a>>, KafkaConsumerError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.message_stream.poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(res)) => match res {
                Err(e) => {
                    warn!("Kafka error when consuming message: {}", e);
                    Poll::Ready(Some(Err(KafkaConsumerError::KafkaError(e))))
                }
                Ok(msg) => match msg.to_event() {
                    Ok(event) => Ok(Envelope {
                        data: event,
                        raw_msg: msg,
                        tx: this.tx.clone(),
                    }),
                    Err(err) => {
                        warn!("CloudEvent error when parsing message: {}", err);
                        Err(KafkaConsumerError::CloudEventError(err))
                    }
                }
                .map(Poll::Ready)
                .map(Some),
            },
        }
    }
}

pub struct DataExchangeKafkaConsumer {
    consumer: StreamConsumer,
}

impl DataExchangeKafkaConsumer {
    pub fn new(consumer: StreamConsumer) -> Self {
        Self { consumer }
    }

    pub fn stream(&self) -> impl Stream<Item = Result<Envelope<Event, BorrowedMessage>, KafkaConsumerError>> + '_ {
        let (tx, rx) = unbounded_channel::<Reply<BorrowedMessage>>();
        let consumer_config = self.consumer.client().config().clone();
        tokio::spawn(async move {
            let consumer: StreamConsumer = consumer_config.create().expect("Failed to create consumer");
            while let Some(reply) = rx.recv().await {
                match reply {
                    Reply::Ok(msg) => {
                        if let Err(e) = consumer.commit_message(&msg, CommitMode::Async) {
                            warn!("Error when committing message: {}", e);
                        }
                    }
                    Reply::Err(e) => error!("Error when consuming message: {}", e),
                }
            }
        });
        KafkaStream::new(self.consumer.stream(), tx)
    }
}
