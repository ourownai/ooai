use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use cloudevents::{Event, EventBuilder, EventBuilderV10};
use futures::Future;
use futures::Stream;
use futures_core::future::LocalBoxFuture;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::mqttbytes::v5::Publish;
use rumqttc::v5::{AsyncClient, ConnectionError, Event as MQTTEvent, EventLoop, Incoming};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tracing::error;

use crate::data_exchange::exchange_core::{Envelope, Reply};
use crate::data_streams::combine::Combine;
use crate::utils::bigboterror::BigbotError;

#[derive(Debug)]
pub enum Error {
    BigbotError(BigbotError),
    ConnectionError(ConnectionError),
    Disconnected,
}

/// A wrapper of [rumqttc], used to implement [Stream] trait
pub struct MQTTStream {
    event_loop: Rc<RefCell<EventLoop>>,
    tx: UnboundedSender<Reply<Publish>>,
    poll_future: Option<LocalBoxFuture<'static, Result<Option<Envelope<Event, Publish>>, Error>>>,
}

async fn event_poll(
    event_loop: Rc<RefCell<EventLoop>>,
    tx: UnboundedSender<Reply<Publish>>,
) -> Result<Option<Envelope<Event, Publish>>, Error> {
    let next = event_loop.borrow_mut().poll().await;
    match next {
        Ok(event) => match event {
            MQTTEvent::Incoming(Incoming::Publish(publish)) => match RawMessage(&publish).try_into() {
                Ok(event) => Ok(Some(Envelope {
                    data: event,
                    raw_msg: publish.clone(),
                    tx,
                })),
                Err(e) => Err(e),
            },
            _ => Ok(None),
        },
        Err(e) => Err(Error::ConnectionError(e)),
    }
}

impl Stream for MQTTStream {
    type Item = Result<Envelope<Event, Publish>, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            match this.poll_future.as_mut() {
                None => {
                    let fut = event_poll(this.event_loop.clone(), this.tx.clone());
                    let boxed_fut = Box::pin(fut);
                    this.poll_future = Some(boxed_fut);
                }
                Some(fut) => {
                    let poll = fut.as_mut().poll(cx);
                    if poll.is_ready() {
                        this.poll_future = None;
                    }
                    match poll {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(Ok(None)) => continue,
                        Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(e))),
                        Poll::Ready(Ok(Some(envelope))) => return Poll::Ready(Some(Ok(envelope))),
                    }
                }
            }
        }
    }
}

impl MQTTStream {
    pub fn new(event_loop: EventLoop, tx: UnboundedSender<Reply<Publish>>) -> Self {
        let boxed = Rc::new(RefCell::new(event_loop));
        Self {
            event_loop: boxed,
            tx,
            poll_future: None,
        }
    }
}

const CONTENT_TYPE: &str = "application/cloudevents+json";
const ID: &str = "CE-id";
const SOURCE: &str = "CE-source";
const TYPE: &str = "CE-type";
const SPEC_VERSION: &str = "CE-specversion";
const TIME: &str = "CE-time";
const DATA_SCHEMA: &str = "CE-dataschema";
const SUBJECT: &str = "CE-subject";
const DATA_CONTENT_TYPE: &str = "CE-datacontenttype";

struct RawMessage<'a>(&'a Publish);

impl From<Box<dyn std::error::Error>> for Error {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Error::BigbotError(BigbotError::UnexpectedError(err.to_string()))
    }
}

impl From<rumqttc::v5::ClientError> for Error {
    fn from(err: rumqttc::v5::ClientError) -> Self {
        match err {
            rumqttc::v5::ClientError::Disconnected => Error::Disconnected,
            _ => Error::ConnectionError(err.into()),
        }
    }
}

impl<'a> TryInto<Event> for RawMessage<'a> {
    type Error = Error;

    fn try_into(self) -> Result<Event, Self::Error> {
        let publish = self.0;
        let mut builder = EventBuilderV10::new();
        let properties = publish.properties.clone().unwrap_or_default();
        let content_type = properties.content_type.unwrap_or(CONTENT_TYPE.to_string());
        let mut headers = properties
            .user_properties
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<HashMap<String, String>>();

        if let Some(id) = headers.remove(ID) {
            builder = builder.id(id);
        }
        if let Some(ce_type) = headers.remove(TYPE) {
            builder = builder.ty(ce_type);
        }
        if let Some(source) = headers.remove(SOURCE) {
            builder = builder.source(source);
        }
        if let Some(time) = headers.remove(TIME) {
            builder = builder.time(time.to_string());
        }
        if let Some(subject) = headers.remove(SUBJECT) {
            builder = builder.subject(subject);
        }
        if let Some(data_schema) = headers.remove(DATA_SCHEMA) {
            builder = builder.data_with_schema(
                content_type,
                data_schema.to_string(),
                publish.payload.to_vec(),
            );
        } else {
            builder = builder.data(content_type, publish.payload.to_vec());
        }
        match String::from_utf8(publish.topic.to_vec()) {
            Ok(topic) => {
                builder = builder.extension("topic", topic.as_str());
            }
            Err(e) => return Err(Error::BigbotError(BigbotError::UnexpectedError(e.to_string()))),
        }
        for (k, v) in headers {
            builder = builder.extension(&k, &v);
        }
        builder.build().map_err(|e| Error::BigbotError(BigbotError::UnexpectedError(e.to_string())))
    }
}

pub struct DataExchangeMQTTStream {
    event_loop: EventLoop,
    async_client: AsyncClient,
}

impl DataExchangeMQTTStream {
    pub fn new(async_client: AsyncClient, event_loop: EventLoop) -> Self {
        Self {
            event_loop,
            async_client,
        }
    }

    pub fn stream(self) -> impl Stream<Item = Result<Envelope<Event, Publish>, Error>> + 'static {
        let (tx, mut rx) = unbounded_channel();
        let DataExchangeMQTTStream {
            event_loop,
            async_client,
        } = self;
        let mqtt_stream = MQTTStream::new(event_loop, tx);

        Combine::new(mqtt_stream, async move {
            while let Some(res) = rx.recv().await {
                match res {
                    Reply::Ok(msg) => {
                        if let Err(e) = async_client.ack(&msg).await {
                            error!("Error when acknowledging message: {}", e);
                        }
                    }
                    Reply::Err(e) => {
                        error!("Error when consuming message: {}", e);
                        break;
                    }
                }
            }
        })
    }

    pub async fn publish(&mut self, topic: &str, payload: &[u8], qos: QoS) -> Result<(), Error> {
        let payload_owned = payload.to_vec();
        self.async_client.publish(topic, qos, false, payload_owned).await.map_err(Error::from)
    } 

    pub async fn next(&mut self) -> Option<Result<Envelope<Event, Publish>, Error>> {
        let next = self.event_loop.poll().await;
        match next {
            Ok(event) => match event {
                MQTTEvent::Incoming(Incoming::Publish(publish)) => match RawMessage(&publish).try_into() {
                    Ok(event) => {
                        let (tx, _) = unbounded_channel();
                        Some(Ok(Envelope {
                            data: event,
                            raw_msg: publish,
                            tx,
                        }))
                    }
                    Err(e) => Some(Err(e)),
                },
                _ => None,
            },
            Err(e) => Some(Err(Error::ConnectionError(e))),
        }
    }
}
