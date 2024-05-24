use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;
use async_trait::async_trait;
use futures::future::try_join_all;

pub mod kafka;
pub mod mock;
pub mod mqtt;

mod combine;

/// Domain-specific error
#[derive(Debug)]
pub enum Error {
    InternalError(Box<dyn std::error::Error + Send + Sync>),
    Cancelled,
    CodecError(serde_json::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InternalError(e) => write!(f, "InternalError: {}", e),
            Error::Cancelled => write!(f, "Cancelled"),
            Error::CodecError(e) => write!(f, "CodecError: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::InternalError(e) => Some(e.deref()),
            Error::Cancelled => None,
            Error::CodecError(e) => Some(e),
        }
    }
}

#[async_trait]
pub trait Sink<T, E> {
    async fn consume(&self, item: T) -> Result<(), E>
    where
        T: 'async_trait;
}

pub struct DrainSink<E>(std::marker::PhantomData<E>);

impl<E> Default for DrainSink<E> {
    fn default() -> DrainSink<E> {
        DrainSink(std::marker::PhantomData)
    }
}

#[async_trait]
impl<T, E> Sink<T, E> for DrainSink<E>
where
    T: Send,
    E: Sync,
{
    async fn consume(&self, _item: T) -> Result<(), E>
    where
        T: 'async_trait,
    {
        Ok(())
    }
}

pub struct MultiSink<T, E> {
    sinks: Vec<Box<dyn Sink<Arc<T>, E> + Send + Sync>>,
}

impl<T, E> MultiSink<T, E> {
    pub fn new(sinks: Vec<Box<dyn Sink<Arc<T>, E> + Send + Sync>>) -> Self {
        Self { sinks }
    }
}

#[async_trait]
impl<'a, T, E> Sink<T, E> for MultiSink<T, E>
where
    T: Send + Sync,
    E: Send,
{
    async fn consume(&self, item: T) -> Result<(), E>
    where
        T: 'async_trait,
    {
        let shared_item = Arc::new(item);
        let fut_vec: Vec<_> = self
            .sinks
            .iter()
            .map(|sink| sink.consume(shared_item.clone()))
            .collect();
        let _ = try_join_all(fut_vec).await?;
        Ok(())
    }
}

pub struct LogSink {
    name: String,
}

impl LogSink {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl<T, E> Sink<T, E> for LogSink
where
    T: Send + Sync + std::fmt::Debug,
    E: Send,
{
    async fn consume(&self, item: T) -> Result<(), E>
    where
        T: 'async_trait,
    {
        println!("{}: {:?}", self.name, item);
        Ok(())
    }
}

#[async_trait]
pub trait Ack {
    async fn ack(&self) -> Result<(), Error>;
}