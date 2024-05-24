use std::cell::Cell;
use std::time::Duration;
use cloudevents::binding::rdkafka::{FutureRecordExt, MessageRecord};
use cloudevents::{Event, EventBuilder, EventBuilderV10};
use futures::StreamExt;
use rdkafka::client::DefaultClientContext;
use rdkafka::config::FromClientConfig;
use rdkafka::consumer::{Consumer, DefaultConsumerContext, StreamConsumer};
use rdkafka::error::KafkaResult;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::{AsyncRuntime, TokioRuntime};
use rdkafka::ClientConfig;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{AsyncClient, MqttOptions};
use tokio::select;
use tokio::time::sleep;
use uuid::Uuid;
use bigbot_rust::data_exchange::exchange::DataExchange;
use bigbot_rust::data_exchange::streams::kafka_sink::KafkaSink;
use bigbot_rust::data_exchange::streams::kafka_stream::{
    DataExchangeKafkaConsumer, KafkaConsumerError,
};
use bigbot_rust::data_exchange::streams::mqtt_stream::DataExchangeMQTTStream;
use bigbot_rust::data_exchange::streams::LogSink;

const KAFKA_BOOTSTRAP_SERVERS: &str = "[::]:19092";
const KAFKA_MESSAGE_TIMEOUT_MS: &str = "5000";
const KAFKA_ACKS: &str = "1";
const KAFKA_GROUP_ID: &str = "test_client";
const MQTT_BROKER_ADDRESS: &str = "[::]";
const MQTT_BROKER_PORT: u16 = 1883;
const MQTT_TOPIC: &str = "test/#";
const KAFKA_TOPIC: &str = "test";
const REPEAT: i32 = 10;

fn create_producer<R: AsyncRuntime>() -> KafkaResult<FutureProducer<DefaultClientContext, R>> {
    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", KAFKA_BOOTSTRAP_SERVERS);
    config.set("message.timeout.ms", KAFKA_MESSAGE_TIMEOUT_MS);
    config.set("acks", KAFKA_ACKS);
    FutureProducer::from_config(&config)
}

fn create_consumer<R: AsyncRuntime>() -> KafkaResult<StreamConsumer<DefaultConsumerContext, R>> {
    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", KAFKA_BOOTSTRAP_SERVERS);
    config.set("group.id", KAFKA_GROUP_ID);
    config.set("message.timeout.ms", KAFKA_MESSAGE_TIMEOUT_MS);
    config.set("acks", KAFKA_ACKS);
    StreamConsumer::from_config(&config)
}

async fn create_mqtt_client(client_id: &str) -> (AsyncClient, rumqttc::v5::EventLoop) {
    let mqtt_options = MqttOptions::new(client_id, MQTT_BROKER_ADDRESS, MQTT_BROKER_PORT);
    AsyncClient::new(mqtt_options, 10)
}

async fn create_event(id: Uuid) -> MessageRecord {
    let evt = EventBuilderV10::new()
        .id(id.to_string())
        .ty("test_event.test_application")
        .source("http://localhost/")
        .extension("someint", "10")
        .data("application/json", serde_json::json!({"hello": "world"}))
        .build()
        .expect("Failed to build event");
    MessageRecord::from_event(evt).expect("Failed to build message record")
}

#[tokio::test]
async fn verify_connection() {
    let (async_client, mut event_loop) = create_mqtt_client("data-exchange").await;
    async_client
        .subscribe(MQTT_TOPIC, QoS::AtLeastOnce)
        .await
        .unwrap();

    let pub_handler = async move {
        for i in 0..20 {
            async_client
                .publish(
                    "test",
                    QoS::AtLeastOnce,
                    false,
                    format!("Hello world {}", i),
                )
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    };

    let sub_handler = async move {
        loop {
            let notification = event_loop.poll().await.unwrap();
            println!("Received = {:?}", notification);
        }
    };

    select! {
        _ = pub_handler => println!("Done"),
        _ = sub_handler => println!("Done"),
    }
}

#[tokio::test]
async fn real_world() {
    let (async_client, event_loop) = create_mqtt_client("data-exchange-1").await;
    async_client
        .subscribe(MQTT_TOPIC, QoS::AtLeastOnce)
        .await
        .unwrap();

    let producer = create_producer().unwrap();
    let mqtt_stream = DataExchangeMQTTStream::new(async_client, event_loop);
    let source = tokio_stream::StreamExt::map(mqtt_stream.stream(), |e| e.unwrap());
    let sink: KafkaSink = KafkaSink::new(producer, KAFKA_TOPIC.to_string());

    // TODO: need refactor to be using Envelop to enable the messages commitable
    let data_exchange: DataExchange<Event, _, _, _> = DataExchange::new(source, sink);

    //TODO: add more control logic and assertions programmatically
    select! {
        _ = data_exchange.start() => println!("Done"),
        _ = tokio::time::sleep(Duration::from_secs(5)) => println!("Done"),
    }
}

#[tokio::test]
async fn mqtt_source() {
    let (async_client, event_loop) = create_mqtt_client("data-exchange-1").await;
    async_client
        .subscribe(MQTT_TOPIC, QoS::AtLeastOnce)
        .await
        .unwrap();

    let source = DataExchangeMQTTStream::new(async_client, event_loop);
    let handle = source.stream().for_each(|e| async move {
        println!("Received = {:?}", e);
    });

    select! {
        _ = handle => println!("Done"),
        _ = tokio::time::sleep(Duration::from_secs(1)) => println!("Done"),
    }
}

#[tokio::test]
async fn producer() {
    let producer = create_producer::<TokioRuntime>().expect("Can't create producer");
    producer
        .send(
            FutureRecord::to(KAFKA_TOPIC).payload("Hello world").key("key"),
            Duration::from_secs(0),
        )
        .await
        .expect("TODO: panic message");
}

#[tokio::test]
async fn consumer() {
    let topics = [KAFKA_TOPIC];
    let consumer = create_consumer::<TokioRuntime>().expect("Can't create consumer");
    consumer
        .subscribe(&topics)
        .expect("Can't subscribe to specified topics");

    let dx_consumer = DataExchangeKafkaConsumer::new(consumer);
    let stream = dx_consumer.stream();
    let received = Cell::new(0);
    let source = stream.filter_map(|e| async {
        if e.is_ok() {
            received.set(received.get() + 1);
        }
        e.ok()
    });

    let log_sink = LogSink::new(KAFKA_TOPIC.to_string());
    let data_exchange: DataExchange<Event, _, _, KafkaConsumerError> =
        DataExchange::new(source, log_sink);

    let mock_producer = async {
        let producer = create_producer::<TokioRuntime>().expect("Can't create producer");
        sleep(Duration::from_millis(100)).await;
        for _ in 0..REPEAT {
            let msg_rec = create_event(Uuid::new_v4()).await;
            let record: FutureRecord<String, Vec<u8>> =
                FutureRecord::to(KAFKA_TOPIC).message_record(&msg_rec);
            producer
                .send_result(record)
                .expect("Failed to send message")
                .await
                .expect("Failed to send message")
                .expect("Failed to send message");
            sleep(Duration::from_millis(100)).await;
        }
        sleep(Duration::from_secs(1)).await;
    };

    select! {
        _ = data_exchange.start() => assert!(false, "Mock stream should finish first"),
        _ = mock_producer => {
            assert!(received.get() == REPEAT, "Should receive all messages, received: {}", received.get())
        },
    }
}