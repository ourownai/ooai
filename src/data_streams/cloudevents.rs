use cloudevents::{Event, EventBuilder, EventBuilderV10};
use rumqttc::{AsyncClient, Event as MqttEvent, MqttOptions, QoS, EventLoop};
use serde_json::Value;
use tracing::error;
use uuid::Uuid;
use rdkafka::producer::{FutureRecord, FutureProducer};

use crate::data_exchange::exchange_core::Sink;
use crate::data_streams::kafka::KafkaSink;

pub struct CloudEventHandler {
    producer: KafkaSink,
    mqtt_client: AsyncClient,
    eventloop: EventLoop,
}

impl CloudEventHandler {
    pub async fn new(kafka_producer: FutureProducer, mqtt_broker: &str, mqtt_port: u16, kafka_topic: String) -> Result<Self, rumqttc::ClientError> {
        let producer = KafkaSink::new(kafka_producer, kafka_topic);
        let mqtt_options = MqttOptions::new(Uuid::new_v4().to_string(), mqtt_broker, mqtt_port);
        let (mqtt_client, eventloop) = AsyncClient::new(mqtt_options, 10);
        Ok(Self {
            producer,
            mqtt_client,
            eventloop,
        })
    }

    pub async fn subscribe(&mut self, topic: &str) -> Result<(), rumqttc::ClientError> {
        self.mqtt_client.subscribe(topic, QoS::AtLeastOnce).await?;
        Ok(())
    }

    pub async fn handle_events(&mut self) {
        loop {
            let event = self.eventloop.poll().await;
            if let Ok(MqttEvent::Incoming(rumqttc::Packet::Publish(p))) = event {
                let bytes = p.payload;
                if let Ok(event) = serde_json::from_slice::<Value>(bytes.as_ref()) {
                    let classification = event.get("classification")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| {
                            error!("Classification extension is missing or not a string");
                            String::new()
                        });

                    let message = event.get("data")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| {
                            error!("Message data is missing or not a string");
                            String::new()
                        });

                    self.route_message(&classification, &message).await;
                } else {
                    error!("Error deserializing CloudEvent");
                }
            }
        }
    }

    pub async fn route_message(&self, classification: &str, message: &str) {
        // Implement message routing logic here
        // Example: Send the message to a Kafka topic based on the classification
        let record = FutureRecord::to(classification).payload(message);
        if let Err(e) = self.producer.consume(record).await {
            error!("Error sending message to Kafka: {:?}", e);
        }
    }

    pub fn create_cloudevent(classification: String, message: String) -> Event {
        let event = EventBuilderV10::new()
            .id(Uuid::new_v4().to_string())
            .source("example.com/message")
            .ty("message.classified")
            .data("text/plain", message.as_bytes())
            .extension("classification", classification)
            .build()
            .unwrap();
        event
    }
}
