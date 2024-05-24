//! This module defines structures and functionalities for the BigBot Data Exchange.
//! It leverages CloudEvents for event management and asynchronous message processing.
//! It uses streams for handling incoming data, a custom Reply type for managing responses,
//! and sinks for processing and routing the data.
//!
//! This module is designed to be highly modular and abstract, allowing for flexible integration
//! with different sources, sinks, and message formats. It emphasizes asynchronous processing
//! and error handling to manage data flows effectively.
//!
//! # Key Components:
//!
//! - `Reply<O>`: A generic Result type used for sending operation outcomes, either Ok(O) or an Error.
//!
//! - `Envelope<T, R>`: A struct representing a message envelope containing the data (`T`),
//! the raw message (`R`), and a transmitter (`tx`) for sending replies.
//!
//! - `DataExchange<I, U, D, E>`: A generic struct designed to abstract the data exchange process.
//! It connects a source of incoming data (`U`) to a data sink (`D`) for processing.
//! The struct is parameterized over:
//! - `I`: An item implementing the `AttributesReader` trait, typically representing CloudEvents.
//! - `U`: A stream of incoming data items wrapped in `Envelope<I, R>`.
//! - `D`: A sink for processing items of type `I` and producing a result or error of type `E`.
//! - `E`: The error type that can be emitted during processing.
//!
//! # Functionality:
//!
//! - `new(source: U, sink: D) -> Self`: Constructs a new instance of `DataExchange` with a given source and sink.
//!
//! - `start(self) -> Result<(), E>`: Starts the data exchange process by consuming items from the source stream,
//! processing them through the sink, and handling success or failure responses. It uses asynchronous
//! stream processing to map each enveloped item to a result and attempts to process each item concurrently.
//!
//! # Utility Functions (placeholders for future implementation):
//!
//! - `parse_message(message: &String) -> EntityGraph`: Parses a raw message string into an `EntityGraph`.
//!
//! - `classify_message(metadata: &HashMap<String, MetadataValue>, entity_graph: &EntityGraph) -> Classification`:
//! Classifies a message based on its metadata and entity graph.
//!
//! - `create_cloudevent(classification: Classification, message: String) -> cloudevents::Event`:
//! Creates a CloudEvent based on the message classification and content.
//!
//! - `classify_and_route_message(message: String, metadata: &MessageMetadata) -> Event`:
//! Classifies a message, routes it accordingly, and wraps the result in a CloudEvent.

use cloudevents::{AttributesReader, Event, EventBuilderV10};
use futures::{Stream, StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::commons::constants::Classification;
use crate::bindings::spacy_bindings::EntityGraph;
use crate::messaging::message_metadata::{MessageMetadata, MetadataValue};
use crate::data_exchange::exchange_interfaces::{ConnectionType, ConnectionInfo};
use crate::data_streams::grpc::DataExchangeImpl;
use std::any::Any;


#[derive(Debug)]
pub struct Envelope<T, R> {
    pub data: T,
    pub raw_msg: R,
    pub(crate) tx: UnboundedSender<Reply<R>>,
}

#[derive(Debug, Error)]
pub enum DataExchangeError {
    #[error("Failed to send reply: {0}")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<Box<dyn Any + Send>>),
    #[error("Sink error: {0}")]
    SinkError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Other error: {0}")]
    Other(Box<dyn Error + Send + Sync>),
}

impl<R> From<tokio::sync::mpsc::error::SendError<Result<R, DataExchangeError>>> for DataExchangeError
where
    R: Send + 'static,
{
    fn from(err: tokio::sync::mpsc::error::SendError<Result<R, DataExchangeError>>) -> Self {
        match err.0 {
            Ok(r) => DataExchangeError::SendError(tokio::sync::mpsc::error::SendError(Ok(Box::new(r) as Box<dyn Any + Send>))),
            Err(e) => e,
        }
    }
}

impl From<Box<dyn Error + Send + Sync>> for DataExchangeError {
    fn from(err: Box<dyn Error + Send + Sync>) -> Self {
        DataExchangeError::Other(err)
    }
}

pub type Reply<O> = Result<O, DataExchangeError>;

// Define a trait for the source of incoming data
pub trait DataExchangeSource<I, R>: Stream<Item = Envelope<I, R>> + Unpin
where
    I: AttributesReader,
{
    fn next_item(&mut self) -> Option<Envelope<I, R>>;
}

pub trait Sink<T> {
    type Error: std::error::Error + Send + Sync + 'static;
    async fn consume(&self, item: T) -> Result<(), Self::Error>;
}

// Define a trait for the sink for processing data items
pub trait DataExchangeSink<I>: Sink<I> + Send + Clone
where
    I: AttributesReader,
{
    fn consume_item(&mut self, item: I) -> Result<(), Self::Error>;
}

// Modify the DataExchange struct to use the DataExchangeSource and DataExchangeSink traits
pub struct DataExchangeStruct<I, S, D, R>
where
    I: AttributesReader,
    S: DataExchangeSource<I, R>,
    D: DataExchangeSink<I>,
    R: Send + 'static,
{
    source: S,
    sink: D,
    _phantom: PhantomData<(I, R)>,
}

// Provide a factory function to create a DataExchange instance
pub fn create_data_exchange<I, S, D, R>(source: S, sink: D) -> DataExchangeStruct<I, S, D, R>
where
    I: AttributesReader,
    S: DataExchangeSource<I, R>,
    D: DataExchangeSink<I>,
    R: Send + 'static,
{
    DataExchangeStruct {
        source,
        sink,
        _phantom: PhantomData,
    }
}

impl<I, R, S, D> DataExchangeStruct<I, S, D, R>
where
    I: AttributesReader + Send + 'static,
    S: DataExchangeSource<I, R>,
    D: DataExchangeSink<I>,
    R: Send + 'static,
{
    pub fn new(source: S, sink: D) -> Self {
        Self {
            source,
            sink,
            _phantom: PhantomData,
        }
    }

    pub async fn start(self) -> Result<(), DataExchangeError> {
        let sink = self.sink;
        self.source
            .map(Ok::<_, DataExchangeError>)
            .try_for_each_concurrent(None, move |envelope| {
                let sink = sink.clone();
                async move {
                    let Envelope { data, raw_msg, tx } = envelope;
                    match sink.consume(data).await {
                        Ok(_) => tx.send(Ok(raw_msg)).map_err(DataExchangeError::from)?,
                        Err(e) => tx
                            .send(Err(DataExchangeError::SinkError(Box::new(e))))
                            .map_err(DataExchangeError::from)?,
                    }
                    Ok(())
                }
            })
            .await
    }

    // Preserved and possibly refactored functionalities for message classification and parsing.
    fn parse_message(message: &str) -> EntityGraph {
        // Implementation detail...
        todo!("Implement parse_message");
    }

    fn classify_message(
        metadata: &HashMap<String, MetadataValue>,
        entity_graph: &EntityGraph,
    ) -> Classification {
        // Implementation detail...
        todo!("Implement classify_message");
    }

    fn create_cloudevent(classification: Classification, message: String) -> Event {
        let cls: &str = classification.into();
        EventBuilderV10::new()
            .id(Uuid::new_v4().to_string())
            .source("example.com/message")
            .ty("message.classified")
            .data("text/plain", message)
            .extension("classification", cls)
            .build()
            .expect("Failed to create CloudEvent")
    }

    pub async fn classify_and_route_message(
        &self,
        message: String,
        metadata: &MessageMetadata,
    ) -> Event {
        let entity_graph = Self::parse_message(&message);
        let classification = Self::classify_message(&metadata.metadata, &entity_graph);
        Self::create_cloudevent(classification, message)
    }
}

// Modify the DataExchangeProcessor struct to hold a map of DataExchangeImpl instances
// keyed by their connection type or identifier
pub struct DataExchangeProcessor {
    data_exchanges: HashMap<ConnectionType, Box<dyn DataExchangeImpl<String, Result<HashMap<String, String>, Box<dyn Error>>>>>,
    connection_info: ConnectionInfo,
}

impl DataExchangeProcessor {
    pub fn new(connection_info: ConnectionInfo) -> Self {
        DataExchangeProcessor {
            data_exchanges: HashMap::new(),
            connection_info,
        }
    }

    // Method to register DataExchangeImpl instances
    pub fn register_data_exchange(
        &mut self,
        connection_type: ConnectionType,
        data_exchange_impl: Box<dyn DataExchangeImpl<String, Result<HashMap<String, String>, Box<dyn Error>>>>,
    ) {
        self.data_exchanges.insert(connection_type, data_exchange_impl);
    }

    // Method to retrieve DataExchangeImpl instances
    pub fn get_data_exchange(
        &self,
        connection_type: &ConnectionType,
    ) -> Option<&Box<dyn DataExchangeImpl<String, Result<HashMap<String, String>, Box<dyn Error>>>>> {
        self.data_exchanges.get(connection_type)
    }

    // Method to initiate data exchange based on the connection type
    pub async fn exchange_data(
        &self,
        connection_type: ConnectionType,
        request: String,
    ) -> Result<HashMap<String, String>, Box<dyn Error>> {
        match self.get_data_exchange(&connection_type) {
            Some(data_exchange_impl) => data_exchange_impl.exchange_data(request).await,
            None => Err(format!("Data exchange implementation not found for connection type: {:?}", connection_type).into()),
        }
    }
}