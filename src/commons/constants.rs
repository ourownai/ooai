const MQTT_CLS: &str = "mqtt";
const KAFKA_CLS: &str = "kafka";
const REST_CLS: &str = "rest";
const GRPC_CLS: &str = "grpc";
const WEBHOOK_CLS: &str = "webhook";

pub const INFERENCE_TOPIC_PREFIX: &str = "inference";

#[derive(Clone, Copy, Debug)]
pub enum Classification {
    Mqtt,
    Kafka,
    Rest,
    Grpc,
    Webhook,
}

impl Into<&str> for Classification {
    fn into(self) -> &'static str {
        match self {
            Classification::Mqtt => MQTT_CLS,
            Classification::Kafka => KAFKA_CLS,
            Classification::Rest => REST_CLS,
            Classification::Grpc => GRPC_CLS,
            Classification::Webhook => WEBHOOK_CLS,
        }
    }
}
