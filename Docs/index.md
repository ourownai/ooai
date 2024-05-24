# BigBot Data Exchange Documentation

## Modules

### Replay Buffer
- **Path:** src/buffers/replay_buffer.rs
- **Description:** Implements a fixed-size buffer for storing experiences in reinforcement learning. Replaces old experiences with new ones when full.

#### Structs
- **ReplayBuffer:** Generic structure for storing experiences with a specified maximum capacity.

#### Methods
- **new:** Constructs a new ReplayBuffer instance.
- **add:** Adds or replaces an experience in the buffer.
- **sample:** Retrieves a random subset of experiences.
- **len:** Returns the number of experiences in the buffer.
- **is_full:** Checks if the buffer is full.
- **clear:** Clears the buffer's contents.

### Payment Buffer
- **Path:** src/buffers/payment_buffer.rs
- **Description:** Manages payment processing for Solana and Polkadot networks.

#### Structs
- **CompressedPayment:** Represents compressed payment information.
- **SolanaPaymentProcessor:** Processes payments on Solana.
- **PolkadotPaymentProcessor:** Processes payments on Polkadot.

#### Methods
- **new:** Initializes a new payment processor.
- **compress_payment:** Compresses a payment for processing.
- **process_compressed_payments:** Placeholder for payment processing logic.

### Base Agent
- **Path:** src/agents/base_agent.rs
- **Description:** Encapsulates state and decision-making logic for an agent in a reinforcement learning environment.

#### Structs
- **Agent:** Holds agent's state and a Q-table for decision-making.

#### Methods
- **new:** Instantiates an Agent object.
- **add_domain/skill/knowledge:** Adds domains, skills, or knowledge to the agent.
- **remove_domain/skill/knowledge:** Removes domains, skills, or knowledge from the agent.
- **has_domain:** Checks if the agent has a given domain.

#### Enums
- **ProviderMetadata, CostPerRequest, DataHandling, PerformanceMetrics:** Enums representing various metadata and metrics for providers.

### Knowledge Agent
- **Path:** src/agents/knowledge_agent.rs
- **Description:** Manages knowledge graphs within specific domains for agents.

#### Structs
- **Agent:** Manages knowledge graphs, domains, and skills.

#### Methods
- **new:** Initializes an Agent with empty state and knowledge graph.
- **add/remove_domain/skill/knowledge:** Manages domains, skills, and knowledge graph.
- **merge:** Merges another agent's information into the current agent.
- **clear:** Clears domains, skills, and knowledge graph.
- **to/from_json:** Serializes and deserializes agent's state.
- **add/remove/get_provider_metadata:** Manages provider metadata.
- **build/update_knowledge_graph:** Builds or updates the knowledge graph.
- **search:** Searches the knowledge graph.
- **summarise:** Generates a summary of the knowledge graph.

#### Enums
- **ProviderMetadata, CostPerRequest, DataHandling, PerformanceMetrics:** Detailed metadata and metrics for providers.

### Q Learning Agent
- **Path:** src/agents/q_learning_agent.rs
- **Description:** Implements a Q-learning algorithm for reinforcement learning.

#### Structs
- **QLearningAgent:** Encapsulates an Agent instance for Q-learning.
- **Experience:** Represents an experience in the replay buffer.

#### Fields
- **agent, gamma, learning_rate, exploration_rate, batch_size, replay_buffer, eligibility_traces, softmax_temp:** Fields for managing Q-learning process.

#### Methods
- **new:** Initializes a QLearningAgent with hyperparameters.
- **choose_action, update_q_values, add_experience, update_exploration_rate, save/load_q_table, reset_state:** Methods for managing the learning process.

### JWK API
- **Path:** src/api/jwk.rs
- **Description:** Manages JSON Web Keys for authentication and authorization.

#### Structs
- **AddJwkReq, UpdateJwkReq:** Request bodies for adding and updating JWKs.

#### Functions
- **jwks, get_jwk, add_jwk, update_jwk:** Endpoints for managing JWKs.

### Message API
- **Path:** src/api/msg.rs
- **Description:** Handles personal identifiable information in messages.

#### Structs
- **PIIMaskReq:** Request body for masking PII.

#### Functions
- **pii_mask, apply_access, pii_unmask:** Endpoints for managing PII in messages.

### BigBot Main
- **Path:** src/bin/bigbot.rs
- **Description:** Main entry point of BigBot, sets up the HTTP server.

#### Structs
- **ServerConfig:** Configuration for the server.

#### Enums
- **BigbotError:** Custom errors for BigBot.

#### Functions
- **main, configure_routes, setup_logger:** Initializes and configures the application.

### Bokeh Bindings
- **Path:** src/bindings/bokeh_bindings.rs
- **Description:** Provides bindings to the Bokeh plotting library in Python for creating interactive visualizations.

#### Functions
- **my_rust_function:** A simple Rust function that returns a string, used as an example for creating a Python module.
- **plot_figure:** Plots a Bokeh figure based on the provided data and chart configuration.
- **find_group:** Finds unique groups in a vector of HashMaps based on a specified key.
- **group_by:** Groups items in a vector of HashMaps based on a specified key.
- **ticker:** Formats a floating-point number as a string with a specific format.
- **group_commons:** (Not yet implemented) Placeholder for a function to find common elements between groups.
- **array_count:** Counts the number of elements in each sub-array of a 2D array.
- **array_sum:** Calculates the sum of elements in each sub-array of a 2D array.
- **array_average:** Calculates the average of elements in each sub-array of a 2D array.
- **linear_scale_mixin:** Applies a linear scaling to an array of values.

#### Structs
- **BOKEH:** A global static variable for the Bokeh Python module, wrapped in a Mutex for thread safety.

#### Python Module
- **myrustlib:** A Python module created using the `py_module_initializer!` macro, which exposes the `my_rust_function` to Python.

### Bokeh Charts
- **Path:** src/bindings/bokeh_charts.rs
- **Description:** Provides functionality for creating and rendering Bokeh charts.

#### Structs
- **Item:** Represents an item for a chart, containing information about the block size and graph type.
- **ChartPlot:** Represents a chart plot, containing the plot type and data.

#### Functions
- **compute_bokeh_chart:** Computes and generates a Bokeh chart based on a list of items and data.
- **append_chart_plot:** Appends a chart plot to an item based on the graph type and data.

### Rust Django Extensor
- **Path:** src/bindings/rust_django_extensor.rs
- **Description:** Facilitates integration of Rust's asynchronous runtime with Python, leveraging `pyo3` and `tokio`.

#### Structs
- **RustTokioRuntime:** Encapsulates a Tokio `Runtime` for use from Python.

#### Enums
- **ExtensionError:** Custom error type for the extension.

### spaCy Bindings
- **Path:** src/bindings/spacy_bindings.rs
- **Description:** Provides bindings to the spaCy library for natural language processing in Python.

#### Functions
- **load_spacy_model:** Loads a spaCy language model.
- **spacy_tokenize:** Tokenizes text using the loaded model.
- **spacy_pos_tag:** Applies part-of-speech tagging to tokens.
- **spacy_ner:** Performs named entity recognition on tokens.
- **spacy_dependency_parse:** Conducts dependency parsing on tokens.
- **spacy_custom_operation:** Executes a custom operation using the model.
- **spacy_visualize_entities:** Visualizes named entities in text.

### Key-Value Store
- **Path:** src/clients/kv.rs
- **Description:** Interface and implementations for key-value storage.

#### Structs
- **PrefixedKVStore:** Adds a prefix to keys in another KVStore.
- **MemoryKVStore:** In-memory key-value store.

#### Traits
- **KVStore:** Interface for a key-value store.

#### Enums
- **BigbotError:** Custom errors for key-value operations.

### Neo4j Client
- **Path:** src/clients/neo4j.rs
- **Description:** Functions for creating Neo4j client connections.

#### Functions
- **new_neo4j_client:** Creates a new client with given credentials.
- **new_neo4j_client_from_env:** Creates a client using environment variables.

### PostgreSQL Client
- **Path:** src/clients/postgres.rs
- **Description:** Functions for PostgreSQL client connections.

#### Functions
- **new_postgres_client:** Creates a new client with a connection string.
- **new_postgres_client_from_env:** Uses an environment variable for the connection string.

### Constants
- **Path:** src/commons/constants.rs
- **Description:** Defines constants and enums used across the application.

#### Enums
- **Classification:** Types of classifications for data exchange.

#### Constants
- **MQTT_CLS, KAFKA_CLS, REST_CLS, GRPC_CLS, WEBHOOK_CLS, INFERENCE_TOPIC_PREFIX:** String constants representing various classifications and prefixes.

### Waiter Utility
- **Path:** src/commons/waiter.rs
- **Description:** Provides utility functions for waiting and retrying operations.

#### Structs
- **Waiter:** Represents a waiter for a service with retry logic.

#### Enums
- **WaiterError:** Errors that can occur while waiting for a service.

#### Functions
- **wait_for_postgres, wait_for_neo4j:** Waits for availability of PostgreSQL or Neo4j servers.


```markdown
### Data Exchange Core
- **Path:** src/data_exchange/data_exchange_core.rs
- **Description:** Defines structures and functionalities for the BigBot Data Exchange, leveraging CloudEvents for event management and asynchronous message processing.

#### Structs
- **Envelop<T, R>:** Represents a message envelope containing data, raw message, and a transmitter for sending replies.
- **DataExchange<I, S, D, E>:** Abstracts the data exchange process, connecting a source of incoming data to a data sink for processing.
- **DataExchangeProcessor:** Holds a map of DataExchangeImpl instances keyed by their connection type or identifier.

#### Enums
- **DataExchangeError:** Custom errors for data exchange operations.

#### Traits
- **DataExchangeSource<I, R>:** Defines the source of incoming data.
- **DataExchangeSink<I, E>:** Defines the sink for processing data items.

#### Type Aliases
- **Reply:** A generic Result type used for sending operation outcomes.

#### Functions
- **create_data_exchange:** Factory function to create a DataExchange instance.
- **parse_message:** Parses a raw message string into an EntityGraph.
- **classify_message:** Classifies a message based on its metadata and entity graph.
- **create_cloudevent:** Creates a CloudEvent based on the message classification and content.
- **classify_and_route_message:** Classifies a message, routes it accordingly, and wraps the result in a CloudEvent.

#### Methods
- **DataExchange::new:** Constructs a new instance of DataExchange with a given source and sink.
- **DataExchange::start:** Starts the data exchange process by consuming items from the source stream, processing them through the sink, and handling responses.
- **DataExchangeProcessor::new:** Initializes a new DataExchangeProcessor with the given connection info.
- **DataExchangeProcessor::register_data_exchange:** Registers a DataExchangeImpl instance for a specific connection type.
- **DataExchangeProcessor::get_data_exchange:** Retrieves a DataExchangeImpl instance for a given connection type.
- **DataExchangeProcessor::exchange_data:** Initiates data exchange based on the connection type and request.

### Data Exchange Interfaces
- **Path:** src/data_exchange/data_exchange_interfaces.rs
- **Description:** Defines traits and structs for different data exchange implementations and connection types.

#### Traits
- **DataExchange<Req, Res>:** Defines the interface for data exchange implementations.
- **DataExchangeImpl<Req, Res>:** Defines the trait for specific data exchange implementations.

#### Structs
- **RestDataExchangeImpl:** Implements the DataExchangeImpl trait for REST data exchange.
- **WebhookDataExchangeImpl:** Implements the DataExchangeImpl trait for webhook data exchange.
- **ConnectionInfo:** Holds connection information for different data exchange types (gRPC, REST, webhook, Kafka).

#### Enums
- **ConnectionType:** Represents different connection types (gRPC, REST, webhook, Kafka).

#### Implementations
- **DataExchangeImpl for RestDataExchangeImpl/WebhookDataExchangeImpl:** Implements the exchange_data method for REST/webhook data exchange.
- **DataExchange for RestDataExchangeImpl/WebhookDataExchangeImpl:** Implements the call method for REST/webhook data exchange.

#### Methods
- **Rest/WebhookDataExchangeImpl::new:** Creates a new instance with the given base URL.
- **WebhookDataExchangeImpl::send_request:** Sends a webhook request to the specified URL with the given data.
- **DataExchangeProcessor::new:** Creates a new instance with the given payment and messaging providers.
- **DataExchangeProcessor::send_message:** Sends a message using the specified messaging provider.
- **DataExchangeProcessor::subscribe_events:** Subscribes to events using the specified messaging provider.

### gRPC Data Exchange
- **Path:** src/data_exchange/grpc_data_exchange.rs
- **Description:** Implements the gRPC data exchange functionality.

#### Structs
- **HelloRequest/Response:** Represents the request/response type for the gRPC service.
- **ConnectionInfo:** Holds the gRPC connection information.
- **GrpcDataExchangeImpl:** Implements the gRPC data exchange functionality.

#### Traits
- **HelloClient:** Defines the gRPC client trait for the HelloService.
- **DataExchange<T, R>:** Defines the data exchange trait.

#### Implementations
- **GrpcDataExchangeImpl:** Implements the exchange_data method for gRPC data exchange.

#### Methods
- **GrpcDataExchangeImpl::new:** Creates a new instance with the given connection info and client factory.
- **GrpcDataExchangeImpl::establish_connection:** Establishes a new gRPC connection.
- **DataExchange::call:** Sends a gRPC request and returns the response.

### Hasura
- **Path:** src/data_exchange/hasura.rs
- **Description:** Defines the GraphQL schema and context for Hasura service APIs.

#### Structs
- **ProviderConfig:** Represents the configuration for a provider.
- **HasuraContext:** Holds context data for Hasura.
- **ArcContextFactory:** Implements the HasuraContextFactory trait.
- **HasuraQueryRoot/EmptyMutation/Subscription:** Represents the root query/mutation/subscription type for Hasura.

#### Traits
- **HasuraContextFactory:** Defines the trait for creating a new HasuraContext instance.

#### Implementations
- **HasuraContextFactory for ArcContextFactory:** Implements the `create_context` method to create a new HasuraContext instance with loaded provider configurations.
- **HasuraQueryRoot:** Implements query fields for payment providers, calendar services, and data exchange initiation.

#### Functions
- **load_provider_configs:** Loads provider configurations from JSON files.
- **create_schema_with_context:** Creates a new GraphQL schema with provider configurations and connection info.

#### Methods
- **ArcContextFactory::create_context:** Creates a new HasuraContext instance with provider configurations.
- **HasuraQueryRoot::payment_provider/calendar_service/data_exchange:** Resolves respective query fields, returning objects or initiating data exchange.

### Kafka Data Exchange
- **Path:** src/data_exchange/kafka_data_exchange.rs
- **Description:** Implements data exchange using Apache Kafka as the messaging system.

#### Structs
- **KafkaDataExchangeImpl:** Represents the Kafka data exchange implementation with producer and consumer instances.

#### Implementations
- **KafkaDataExchangeImpl:** Handles message sending and receiving with Kafka.

#### Methods
- **KafkaDataExchangeImpl::new:** Creates a new instance with producer and consumer configurations, topic, and group ID.
- **KafkaDataExchangeImpl::send_message_with_headers:** Sends a message with headers to a specified Kafka topic.
- **KafkaDataExchangeImpl::receive_message_with_headers:** Receives a message with headers from a specified Kafka topic.
- **DataExchange::call:** Sends and receives messages with headers via Kafka, returning the response.

#### Functions
- **dynamic_topic:** Generates dynamic Kafka topic names based on prefix and operator ID.

#### Constants
- **INFERENCE_TOPIC_PREFIX:** Prefix for inference-related Kafka topics.

#### Dependencies
- **async_trait:** Enables asynchronous trait methods.
- **rdkafka:** Rust client library for Apache Kafka.
- **std::collections::HashMap:** Used for storing and retrieving message headers.
- **std::time::Duration:** Specifies timeouts and durations in Kafka operations.

### Request Adapter
- **Path:** src/data_exchange/request_adapter.rs
- **Description:** Demonstrates using the Serde library for deserializing JSON data into structs and extracting relevant data.

#### Structs
- **Pair:** Represents a key-value pair in the JSON data.
- **AdaptiveData:** Structure of the incoming JSON data, containing an ID, name, and a vector of key-value pairs.
- **PaymentTransaction:** Represents a payment transaction extracted from AdaptiveData, with fields like amount, recipient, description, date, etc.

#### Enums
- **DecodeError:** Custom error type for decoding errors, including missing and invalid fields.

#### Implementations
- **TryFrom for PaymentTransaction:** Converts an AdaptiveData instance into a PaymentTransaction instance.

#### Functions
- **run_request_adapter_example:** Demonstrates deserializing JSON data into an AdaptiveData instance and transforming it into a PaymentTransaction instance.

#### Macros
- **get_field:** Simplifies extracting fields from AdaptiveData's key-value pairs and handling missing or invalid fields.

#### Dependencies
- **std::collections::HashMap:** For storing and retrieving key-value pairs.
- **chrono::NaiveDate:** Represents dates without timezone information.
- **serde::{Deserialize, Serialize}:** For serializing and deserializing JSON data.
- **serde_json::Value:** Represents arbitrary JSON values.

### Topics
- **Path:** src/data_exchange/topics.rs
- **Description:** Defines constants for various Kafka topics used in the data exchange system.

#### Constants
- **TOPIC_PAYMENT_INITIATE:** Initiating payment transactions.
- **TOPIC_PAYMENT_COMPLETE:** Completed payment transactions.
- **TOPIC_PAYMENT_FAILED:** Failed payment transactions.
- **TOPIC_PAYMENT_REFUND:** Payment refund requests.
- **TOPIC_PAYMENT_REFUND_COMPLETE:** Completed payment refunds.
- **TOPIC_PAYMENT_REFUND_FAILED:** Failed payment refunds.
- **TOPIC_PAYMENT_QUERY:** Payment query requests.
- **TOPIC_PAYMENT_QUERY_RESPONSE:** Payment query responses.
- **TOPIC_CALENDAR_CREATE_EVENT:** Creating calendar events.
- **TOPIC_CALENDAR_UPDATE_EVENT:** Updating calendar events.
- **TOPIC_CALENDAR_DELETE_EVENT:** Deleting calendar events.
- **TOPIC_CALENDAR_QUERY_EVENTS:** Querying calendar events.
- **TOPIC_CALENDAR_QUERY_EVENTS_RESPONSE:** Calendar event query responses.
- **TOPIC_DATA_EXCHANGE_REQUEST:** Data exchange requests.
- **TOPIC_DATA_EXCHANGE_RESPONSE:** Data exchange responses.
- **TOPIC_INFERENCE_REQUEST:** Inference requests.
- **TOPIC_INFERENCE_RESPONSE:** Inference responses.
- **TOPIC_TRAINING_REQUEST:** Training requests.
- **TOPIC_TRAINING_RESPONSE:** Training responses.
- **TOPIC_MONITORING_METRICS:** Monitoring metrics.
- **TOPIC_MONITORING_ALERTS:** Monitoring alerts.
- **TOPIC_LOGGING_INFO:** Info-level log messages.
- **TOPIC_LOGGING_WARNING:** Warning-level log messages.
- **TOPIC_LOGGING_ERROR:** Error-level log messages.

### CloudEvents Handler
- **Path:** src/data_streams/cloudevents.rs
- **Description:** Implements a CloudEvents handler for consuming events from an MQTT broker and routing them to Kafka topics based on their classification.

#### Structs
- **CloudEventHandler:** Manages the Kafka producer and MQTT client.

#### Implementations
- **CloudEventHandler:** Handles subscription to MQTT topics, event handling, and message routing.

#### Methods
- **new:** Creates a new CloudEventHandler instance.
- **subscribe:** Subscribes to an MQTT topic.
- **handle_events:** Handles incoming MQTT events, converts them to CloudEvents, and routes them to Kafka.
- **route_message:** Routes messages to Kafka based on classification.
- **create_cloudevent:** Creates a CloudEvent from a message.

#### Dependencies
- **cloudevents:** For creating and managing CloudEvents.
- **kafka_producer:** To produce messages to Kafka topics.
- **rumqttc:** For MQTT client functionality.
- **serde_json:** For JSON serialization and deserialization.
- **tracing:** For error logging.
- **uuid:** For generating unique CloudEvent IDs.

### Stream Combinator
- **Path:** src/data_streams/combine.rs
- **Description:** Combines a stream of items with a future, yielding items until the future completes.

#### Structs
- **Combine<T, S, F>:** Combines a stream (S) with a future (F) yielding items of type T.

#### Implementations
- **Combine:** Combines streams and futures.

#### Methods
- **new:** Creates a new Combine instance with a stream and future.
- **poll_next:** Polls both the future and the stream for the next item.

#### Dependencies
- **futures_core:** For Stream trait and types.
- **pin_project_lite:** For pinning stream and future fields.

### Kafka Data Streams
- **Path:** src/data_streams/kafka.rs
- **Description:** Manages consuming and producing messages with Kafka, including converting messages to CloudEvents.

#### Structs
- **KafkaSink:** Wraps FutureProducer for Kafka message production.
- **KafkaStream:** Streams messages from Kafka, converting them into CloudEvents.
- **DataExchangeKafkaConsumer:** Consumes Kafka messages for data exchange.

#### Enums
- **KafkaConsumerError:** Errors during Kafka message consumption and CloudEvent conversion.

#### Implementations
- **KafkaSink:** Implements the Sink trait for Kafka message production.
- **KafkaStream:** Implements the Stream trait for message consumption and conversion.
- **DataExchangeKafkaConsumer:** Manages Kafka message consumption and processing.

#### Methods
- **new:** Creates new instances for KafkaSink, KafkaStream, and DataExchangeKafkaConsumer.
- **stream:** For DataExchangeKafkaConsumer, returns a stream of enveloped events or errors.

#### Dependencies
- **async_trait:** For asynchronous trait definitions.
- **cloudevents:** For CloudEvents functionality.
- **rdkafka:** For Kafka consumer and producer functionality.
- **serde:** For serialization.
- **tokio:** For asynchronous programming.
- **tracing:** For logging.

### Mock Data Stream
- **Path:** src/data_streams/mock.rs
- **Description:** Emits mock data at specified intervals as a Stream.

#### Structs
- **MockSource:** Emits unit items at specified intervals.

#### Implementations
- **MockSource:** Implements the Stream trait.

#### Methods
- **new:** Creates a new MockSource instance.
- **with_interval:** Sets the interval for the mock data emission.

#### Dependencies
- **tokio::time:** For handling time intervals.
- **tokio_stream::Stream:** For defining the Stream trait.

### MQTT Data Stream
- **Path:** src/data_streams/mqtt.rs
- **Description:** Consumes MQTT messages as a Stream and converts them into CloudEvents.

#### Structs
- **MQTTStream:** Wraps rumqttc::EventLoop for MQTT message consumption.
- **DataExchangeMQTTStream:** Manages MQTT message consumption for data exchange.
- **RawMessage:** For converting MQTT messages into CloudEvents.

#### Enums
- **Error:** Types of errors for MQTT stream operations.

#### Implementations
- **MQTTStream:** Implements the Stream trait for MQTT message consumption.
- **DataExchangeMQTTStream:** Manages message acknowledgment and error logging.

#### Functions
- **event_poll:** Polls MQTT events and converts them into CloudEvents.

#### Dependencies
- **cloudevents:** For CloudEvents manipulation.
- **rumqttc:** For MQTT client functionality.
- **serde_json:** For JSON conversion.
- **tracing:** For error logging.

### Messaging
- **Path:** src/messaging/messaging.rs
- **Description:** Defines traits and structs for messaging functionality, focusing on message providers and handlers for different protocols.

#### Traits
- **MessageProvider:** Interface for sending and receiving messages.
- **MessageHandler:** Interface for processing received messages.

#### Structs
- **KafkaMessageProvider:** Handles Kafka-based messaging.
- **MqttMessageProvider:** Manages MQTT-based messaging.
- **WebhookMessageProvider:** Manages webhook-based messaging.
- **MessageHandlerImpl:** Processes received messages.

#### Implementations
- **KafkaMessageProvider:**
  - **new:** Initializes with Kafka configuration.
  - **send_message:** Sends messages to a Kafka topic.
  - **subscribe:** Subscribes to Kafka topics, returning a stream of messages.
- **MqttMessageProvider:**
  - **new:** Initializes with MQTT configuration.
  - **send_message:** Publishes messages to an MQTT topic.
  - **subscribe:** Subscribes to MQTT topics, returning a stream of messages.
- **WebhookMessageProvider:**
  - **new:** Initializes with webhook configuration.
  - **send_message:** Sends messages to a specified webhook URL.
  - **subscribe:** Handles incoming webhook requests, returning a stream of messages.
- **MessageHandlerImpl:**
  - **new:** Creates a new instance for message processing.
  - **handle_message:** Processes received messages and takes appropriate actions.

#### Dependencies
- **async_trait:** For defining asynchronous traits.
- **futures:** For handling asynchronous streams and futures.
- **serde:** For message serialization and deserialization.
- **tokio:** For the asynchronous runtime and networking capabilities.
- **tracing:** For logging and tracing activities.


### Consensus Layer
- **Path:** src/messaging/consensus.rs
- **Description:** Implements the consensus layer for message validation, replication, and synchronization.

#### Structs
- **ConsensusLayer:** Manages the consensus layer components like the TiKV client, local storage, and synchronization mechanisms.

#### Implementations
- **ConsensusLayer:**
  - **new:** Initializes the consensus layer with TiKV endpoints, local storage path, and distributed hash endpoints.
  - **validate_message:** Validates a message by checking its hash and using zero-knowledge proofs.
  - **replicate_message:** Stores a message in the TiKV cluster.
  - **sync_messages:** Synchronizes messages across the network.

#### Methods
- **ConsensusLayer::new:** Sets up the consensus layer with necessary components.
- **ConsensusLayer::validate_message:** Ensures message integrity and authenticity.
- **ConsensusLayer::replicate_message:** Replicates messages for durability.
- **ConsensusLayer::sync_messages:** Keeps the message store consistent across nodes.

#### Dependencies
- **chrono:** For handling timestamps.
- **serde:** For data serialization and deserialization.
- **std::collections::HashMap:** For managing key-value pairs.
- **tikv_client:** For interacting with the TiKV cluster.
- **uuid:** For generating unique message IDs.
- **zkp_library:** For zero-knowledge proof functionality.
- **distributed_hash, local_storage, messaging, replication, sync:** Modules for distributed hash management, local storage, message handling, replication, and synchronization.


# Encryption Module

## Path: src/encryption/encryption.rs
- **Description:** Provides encryption and key management functionalities using the Ockam Vault.

### Structs
- **EncryptHandler:** Handles encryption operations and key management.
  - `vault`: An instance of the Ockam Vault for secure key storage and cryptographic operations.
  - `keyid_store`: A key-value store for storing key IDs.
- **KeysStore:** Implements the ockam_node::KeyValueStorage trait for storing keys in a key-value store.
  - `store`: An instance of a key-value store.

### Implementations
#### KeysStore:
- **new:** Creates a new instance of KeysStore with the given key-value store.
- **put:** Stores a key-value pair in the underlying key-value store.
- **get:** Retrieves a value associated with a given key from the key-value store.
- **delete:** Deletes a key-value pair from the key-value store.
- **keys:** Retrieves all the keys stored in the key-value store.

#### EncryptHandler:
- **new:** Creates a new instance of EncryptHandler with the given KeysStore and key ID store.
- **new1:** Creates a new instance of EncryptHandler with a single key-value store, automatically creating prefixed stores for keys and key IDs.
- **get_or_create_keyid:** Retrieves or creates a key ID for a given user ID and secret attributes.
- **negotiate_shared_keyid:** Negotiates a shared key ID between two users using the Diffie-Hellman key exchange.
- **aes_encrypt_message:** Encrypts a message using AES-GCM with the specified key ID, plaintext, and additional authenticated data (AAD).
- **aes_decrypt_message:** Decrypts an AES-GCM encrypted message using the specified key ID.
- **multisig_key_exchange:** Performs a multi-signature key exchange between multiple users.
- **encrypt_message_for_users:** Encrypts a message for multiple users using their shared keys.
- **decrypt_message_with_shared_key:** Decrypts a message using a shared key between two users.

### Dependencies
- **ockam:** Provides the Ockam Vault for secure key storage and cryptographic operations.
- **serde:** Used for serializing and deserializing data.
- **serde_json:** Used for JSON serialization and deserialization.
- **base64:** Used for encoding and decoding binary data in base64 format.
- **async_trait:** Used for defining asynchronous traits.
- **kafka:** Used for Kafka producer functionality.
- **rand:** Used for generating random values.

### Tests
- **test_encrypt_handler:** Tests the functionality of the EncryptHandler by creating key IDs, encrypting and decrypting messages, and performing multi-signature key exchanges.

---

# Outgoing Message Filter

## Path: src/filters/outgoing_filter.rs
- **Description:** Defines a MessageFilter struct that can be used to filter and paraphrase text messages based on a blacklist of prohibited terms with similarity scores, a whitelist of allowed terms, and a language model from the Spacy library.

### Structs
- **MessageFilter:** Represents the message filter, which contains a blacklist, a whitelist, and a Spacy language model.

### Implementations
#### MessageFilter:
- **new:** Constructs a new MessageFilter instance given a blacklist, a whitelist, and a language model.
- **filter_message:** Filters a message by replacing blacklisted words and their similar tokens with asterisks (*****), based on a given similarity threshold.
- **paraphrase_message:** Paraphrases a message by replacing blacklisted words with similar tokens while keeping the allowed terms unchanged.
- **get_similar_tokens:** Returns a HashMap of tokens similar to the blacklisted terms, based on a similarity threshold.
- **mask_tokens:** Replaces blacklisted terms and their similar tokens with asterisks (*****) in a message, given a similarity threshold.
- **paraphrase_tokens:** Replaces blacklisted terms in a message with similar tokens while keeping the allowed terms unchanged.
- **get_similar_token:** Returns a similar token to a given token, based on a similarity threshold.

### Dependencies
- **std::collections::{HashMap, HashSet}:** Used for storing the blacklist, whitelist, and similar tokens.
- **spacy::Spacy:** Used for initializing the Spacy language model and performing similarity calculations.

---

# Flow Blocks

## Path: src/flows/blocks.rs
- **Description

:** Defines the building blocks for creating and managing flows in the BigBot system.

### Structs
- **FlowBlock:** Represents a single block in a flow, containing an ID, type, label, action, transitions, and error transitions.
  - `id`: A unique identifier for the block.
  - `type`: The type of the block (e.g., "start", "end", "action", "decision").
  - `label`: A human-readable label for the block.
  - `action`: An optional action associated with the block, represented as a string.
  - `transitions`: A vector of transitions to other blocks based on the outcome of the current block.
  - `error_transitions`: A vector of transitions to other blocks in case of an error.
- **FlowBlockTransition:** Represents a transition from one block to another based on a condition.
  - `condition`: The condition that determines whether the transition should be followed, represented as a string.
  - `target_block_id`: The ID of the target block to transition to if the condition is met.
- **FlowGraph:** Represents a directed graph of flow blocks, allowing for the creation and traversal of flows.
  - `blocks`: A vector of FlowBlock instances that make up the flow graph.

### Implementations
#### FlowBlock:
- **new:** Creates a new instance of FlowBlock with the given ID, type, label, action, transitions, and error transitions.
- **add_transition:** Adds a new transition to the block with the given condition and target block ID.
- **add_error_transition:** Adds a new error transition to the block with the given target block ID.

#### FlowGraph:
- **new:** Creates a new instance of FlowGraph with an empty vector of blocks.
- **add_block:** Adds a new FlowBlock to the graph.
- **get_block_by_id:** Retrieves a reference to a FlowBlock with the given ID, if it exists in the graph.
- **get_start_block:** Retrieves a reference to the start block of the flow graph, if it exists.
- **get_end_block:** Retrieves a reference to the end block of the flow graph, if it exists.
- **get_next_block:** Retrieves a reference to the next block based on the current block and the evaluation of its transitions.
- **execute_flow:** Executes the flow graph by traversing the blocks from start to end, evaluating conditions and executing actions along the way.

### Methods
#### FlowBlock:
- **new:** Instantiates a new FlowBlock with specified characteristics like ID, type, label, action, transitions, and error transitions. This allows for a clear definition and construction of individual blocks within a flow.
- **add_transition:** Appends a new transition to a FlowBlock, specifying the condition under which this transition becomes valid and the target block ID to transition to. This method enhances the block's capability to navigate to different parts of the flow based on conditions.
- **add_error_transition:** Incorporates a new error transition into a FlowBlock, targeting a specific block ID for cases when errors occur. This ensures that flows can gracefully handle errors by moving to designated error-handling blocks.

#### FlowGraph:
- **new:** Creates a new, empty instance of FlowGraph. This foundational method sets up a blank canvas for constructing complex flows by adding blocks and defining transitions.
- **add_block:** Introduces a new FlowBlock into the FlowGraph. This method is critical for building the structure of a flow, block by block, including defining their actions, transitions, and error-handling paths.
- **get_block_by_id:** Fetches a reference to a FlowBlock within the graph based on its ID. This lookup method is essential for navigating the flow and accessing blocks for further processing or analysis.
- **get_start_block:** Identifies and returns a reference to the starting block of the flow, typically used to initiate the execution of the flow.
- **get_end_block:** Identifies and returns a reference to the end block of the flow, helping to determine when the flow has reached its conclusion.
- **get_next_block:** Determines the next block to transition to based on the current block's outcomes and transitions. This method is crucial for flow execution, guiding the traversal from one block to the next.
- **execute_flow:** Triggers the execution of the flow, starting from the initial block and progressing through the flow based on the defined transitions and conditions. This method embodies the dynamic aspect of flows, enabling the automated processing and handling of tasks within the BigBot system.

### Methods
#### FlowBlock:
- **new:** Instantiates a new FlowBlock with specified characteristics like ID, type, label, action, transitions, and error transitions. This allows for a clear definition and construction of individual blocks within a flow.
- **add_transition:** Appends a new transition to a FlowBlock, specifying the condition under which this transition becomes valid and the target block ID to transition to. This method enhances the block's capability to navigate to different parts of the flow based on conditions.
- **add_error_transition:** Incorporates a new error transition into a FlowBlock, targeting a specific block ID for cases when errors occur. This ensures that flows can gracefully handle errors by moving to designated error-handling blocks.

#### FlowGraph:
- **new:** Creates a new, empty instance of FlowGraph. This foundational method sets up a blank canvas for constructing complex flows by adding blocks and defining transitions.
- **add_block:** Introduces a new FlowBlock into the FlowGraph. This method is critical for building the structure of a flow, block by block, including defining their actions, transitions, and error-handling paths.
- **get_block_by_id:** Fetches a reference to a FlowBlock within the graph based on its ID. This lookup method is essential for navigating the flow and accessing blocks for further processing or analysis.
- **get_start_block:** Identifies and returns a reference to the starting block of the flow, typically used to initiate the execution of the flow.
- **get_end_block:** Identifies and returns a reference to the end block of the flow, helping to determine when the flow has reached its conclusion.
- **get_next_block:** Determines the next block to transition to based on the current block's outcomes and transitions. This method is crucial for flow execution, guiding the traversal from one block to the next.
- **execute_flow:** Triggers the execution of the flow, starting from the initial block and progressing through the flow based on the defined transitions and conditions. This method embodies the dynamic aspect of flows, enabling the automated processing and handling of tasks within the BigBot system.


# Delegate Graph Documentation

## Delegate Graph
- **Path:** src/graphs/delegate_graph.rs
- **Description:** Defines a system for analyzing and organizing information from natural language input into a structured format, focusing on attributes and connections within a delegate model.

### Structs
- **Attribute:** Represents a characteristic or quality of a delegate, identified by a name and associated with a set of values.
  - `name`: The name of the attribute.
  - `values`: A HashSet of string values associated with the attribute.
- **Delegate:** Represents an entity with a collection of attributes and connections to other entities.
  - `attributes`: A HashMap mapping attribute names to Attribute structs.
  - `connections`: A HashMap mapping entity names to HashSets of connected entity names.

### Implementations
- **Attribute:**
  - `new`: Creates a new instance of Attribute with the given name.
  - `add_value`: Adds a new value to the attribute's values set.
- **Delegate:**
  - `new`: Creates a new instance of Delegate.
  - `add_attribute_value`: Adds a new value to a specified attribute of the delegate. If the attribute does not exist, it is created.
  - `has_attribute_value`: Checks if a specific value is associated with a given attribute.
  - `build_network`: Processes an input string to extract and organize information into attributes and connections.
  - `process_attributes`: Identifies and categorizes attribute values based on predefined prefixes (e.g., "interest:" or "expertise:").
  - `process_connections`: Scans the input to construct a network of connections between non-attribute entities, applying logic to identify relationships marked by specific tokens (e.g., "->").

### Methods
- **Attribute::new**: Creates a new instance of Attribute with the given name.
- **Attribute::add_value**: Adds a new value to the attribute's values set.
- **Delegate::new**: Creates a new instance of Delegate.
- **Delegate::add_attribute_value**: Adds a new value to a specified attribute of the delegate. If the attribute does not exist, it is created.
- **Delegate::has_attribute_value**: Checks if a specific value is associated with a given attribute.
- **Delegate::build_network**: Processes an input string to extract and organize information into attributes and connections.
- **Delegate::process_attributes**: Identifies and categorizes attribute values based on predefined prefixes (e.g., "interest:" or "expertise:").
- **Delegate::process_connections**: Scans the input to construct a network of connections between non-attribute entities, applying logic to identify relationships marked by specific tokens (e.g., "->").

### Key Functionalities
- Extracting and storing attributes and their values from natural language input.
- Building a network of connections between entities based on the input text.
- Efficient storage and retrieval of attributes and connections using HashMaps and HashSets.
- Modular design with separate methods for processing attributes and connections.

### Enhancements
- Refactored for improved modularity and readability.
- Optimized parsing strategies and use of windowed iteration for connection processing.
- Efficient data structures (HashMaps and HashSets) for storing and accessing attributes and connections.

## Event Graph
- **Path:** src/graphs/event_graph.rs
- **Description:** Defines the Event struct and related functionality for managing events in a graph database using Neo4j.

### Structs
- **Event:** Represents an event with properties such as ID, event type, location, significance, duration, and dependencies.
- **Location:** Represents a location with x, y, and z coordinates.
- **Duration:** Represents a duration with start and end timestamps.
- **EventHandler:** Handles operations related to events in the graph database.

### Enums
- **EventHandlerError:** Defines custom error types for event handling operations.

### Implementations
- **Event:**
  - `new`: Creates a new instance of Event with the given properties.
  - `id`: Returns the ID of the event.
  - `significance`: Returns the significance of the event.
  - `is_schedulable`: Checks if the event is schedulable based on its dependencies.
- **Location:**
  - `new`: Creates a new instance of Location with the given coordinates.
  - `distance`: Calculates the distance between two locations.
  - Implements the From trait to create a Location from a tuple of coordinates.
  - Implements the TryFrom trait to create a Location from a string representation of coordinates.
  - Implements the From trait to convert a Location to a string representation.
- **EventHandler:**
  - `new`: Creates a new instance of EventHandler with the given Neo4j graph client.
  - `add_new_event`: Adds a new event to the graph database, creating nodes and relationships for the event and its dependencies.
  - `create_event_node`: Creates a new event node in the graph database with the given properties.
  - `link_events`: Creates a DEPENDS_ON relationship between two events in the graph database.
  - `link_event_to_entity`: Creates a RELATED_TO relationship between an event and an entity in the graph database.

### Functions
- **setup_graph_client:** Sets up the Neo4j graph client for testing purposes.

### Tests
- **test_location_distance:** Tests the distance calculation between two locations.
- **test_event_is_schedulable:** Tests the `is_schedulable` method of the Event struct.
- **test_add_new_event:** Tests the `add_new_event` method of the EventHandler struct by creating a main event with dependencies and verifying the created nodes and relationships in the graph database.

### Dependencies
- **neo4rs:** Used for interacting with the Neo4j graph database.
- **thiserror:** Used for defining custom error types.
- **tokio:** Used for asynchronous programming and testing.
- **std::sync::Arc:** Used for thread-safe reference counting.
- **std::env:** Used for retrieving environment variables during testing.

# Message Entity Graph Documentation

## Path
src/graphs/message_entity_graph.rs

## Description
Implements a graph-based representation and processing of user preferences and entities using a Neo4j database.

### Structs
- **PreferenceGraphHandler:** Represents a handler for managing user preferences within a Neo4j graph database.
  - **neo_client:** An Arc-wrapped Neo4j graph client for database interactions.
- **UserPreference:** Represents a user preference, including the user ID, related entity, timestamp, and preference score.
  - **user_id:** The ID of the user associated with the preference.
  - **entity:** The entity related to the user preference.
  - **ts:** The timestamp of the preference.
  - **score:** The preference score, indicating the user's sentiment towards the entity.

### Constants
- **LINK_USER_WITH_ENTITY_QUERY:** A Cypher query for linking a user with an entity in the graph database.
- **QUERY_USER_PREFERENCES_QUERY:** A Cypher query for retrieving a user's preferences from the graph database.
- **QUERY_ENTITY_USERS_QUERY:** A Cypher query for finding users interested in a specific entity based on their preferences.
- **QUERY_TOP_ENTITIES_QUERY:** A Cypher query for retrieving the top N entities based on the number of user mentions.

### Implementations
- **PreferenceGraphHandler:**
  - **new:** Constructs a new handler with the given Neo4j graph client.
  - **handle:** Processes a user's utterance to extract entities, analyze sentiment, and store them as preferences in the graph database.
  - **query_user_preferences:** Queries the database for a user's preferences and returns them.
  - **query_entity_users:** Finds users interested in a specific entity based on their preferences.
  - **query_top_entities:** Retrieves the top N entities based on the number of user mentions.
  - **compute_user_similarity:** Computes the similarity between two users based on their preferences using cosine similarity.

### Functions
- **extract_entities_and_sentiment:** Extracts entities and analyzes sentiment from a given utterance using the SPACY library.

### Dependencies
- **spacy_bindings:** Provides bindings to the SPACY library for entity extraction and sentiment analysis.
- **BigbotError:** Custom error type for error handling.
- **cpython:** Provides Python integration for Rust.
- **neo4rs:** A Neo4j driver for Rust, used for graph database interactions.
- **std::collections::HashMap:** Used for mapping entities to preference scores.
- **std::sync::Arc:** Used for thread-safe reference counting of the Neo4j graph client.
- **std::str::FromStr:** Used for parsing entity labels from strings.

### Tests
- **test_handle_user_utterance:** Tests the functionality of handling a user utterance and storing the extracted entities and sentiment in the graph database. Additional tests for query_user_preferences, query_entity_users, query_top_entities, and compute_user_similarity can be added to ensure the correctness of these methods.


# Natural Language to Graph Conversion Documentation

## Modules

### Natural Language to Graph Conversion
- **Path:** src/graphs/nl_to_graph.rs
- **Description:** This module provides functionality to process natural language utterances and convert them into structured GraphQL query strings. It leverages the spaCy library for natural language processing tasks such as entity recognition and part-of-speech tagging.

#### Structs
- **QueryMapping:** Represents the central structure that holds the mappings of entities and slots identified in an utterance to their corresponding values. It contains two fields:
  - **entity_map:** A HashMap that maps recognized entity types (e.g., PERSON, ORG) to their corresponding GraphQL query fields. This facilitates the construction of the part of the GraphQL query that queries specific entities.
  - **slot_map:** A HashMap that maps slot names derived from the utterance (e.g., nouns, verbs, adjectives) to their values. This aids in the dynamic construction of the query based on the utterance content.

#### Implementations
- **QueryMapping:**
  - **new:** Constructor for creating a new instance of QueryMapping with empty entity_map and slot_map.
  - **add_entity:** Adds an identified entity and its value to the entity_map.
  - **add_slot:** Adds a detected slot and its value to the slot_map.
  - **merge:** Merges another QueryMapping instance into the current one by extending the entity_map and slot_map.

#### Functions
- **utterance_to_query_mapping:** Takes a natural language utterance as input and returns a QueryMapping instance. It processes the utterance using the spaCy model to extract entities and slots, populating the QueryMapping with this information.
- **get_entity_mapping:** Returns a predefined mapping of spaCy entity labels to their corresponding GraphQL query fields.
- **process_tokens_for_slots:** Identifies slots from the tokens in the processed utterance and adds them to the QueryMapping. It handles nouns, proper nouns, verbs with direct objects, and adjectives modifying nouns.
- **generate_query_from_mapping:** Generates a GraphQL query string from the QueryMapping and sends it to a GraphQL endpoint. It constructs the query string based on the entity and slot mappings and sends the query using an HTTP client.
- **construct_query_string:** Constructs the GraphQL query string from the entity and slot mappings in the QueryMapping. It iterates over the mappings and builds the appropriate query parts for entities and slots.
- **construct_entity_queries:** Adds query parts for each entity in the entity_map to the query string.
- **construct_slot_queries:** Adds query parts for each slot in the slot_map to the query string, avoiding redundancy with entities.
- **send_query:** Sends the constructed GraphQL query to the specified endpoint using an HTTP client and returns the response.
- **parse_response:** Parses the JSON response received from the GraphQL endpoint and extracts the relevant information based on the expected response structure.

#### Dependencies
- **spacy:** Used for natural language processing tasks such as entity recognition and part-of-speech tagging.
- **std::collections::HashMap:** Used for storing the entity and slot mappings in the QueryMapping struct.
- **reqwest:** Used for sending HTTP requests to the GraphQL endpoint.
- **serde_json:** Used for parsing the JSON response received from the GraphQL endpoint.


# Identity Graph Documentation

## Identity Graph
- **Path:** src/graphs/identity_graph.rs
- **Description:** Implements an identity graph structure and provides functionality for managing identities and their relationships.

### Structs
- **Identity:** Represents an identity associated with a user, containing the user ID, identity type, and timestamp.
- **IdentityGraph:** Represents the identity graph structure, containing a map of identity nodes and a vector of identity edges.
- **IdentityNode:** Represents a node in the identity graph, containing the user ID and a map of attributes.
- **IdentityEdge:** Represents an edge in the identity graph, connecting two identity nodes with a relationship.
- **IdentityGraphHandler:** Handles interactions with the identity graph in the Neo4j database.

### Enums
- **IdentityGraphError:** Defines custom error types for identity graph operations.
- **IdentityType:** Represents different types of identities (phone number, phone identifier, email, user agent, IP).

### Implementations

#### IdentityType
- **as_tuple:** Returns the identity type as a tuple of its label and value.

#### IdentityGraph
- **new:** Creates a new instance of IdentityGraph.
- **add_node:** Adds a new identity node to the graph.
- **add_edge:** Adds a new identity edge to the graph.
- **get_node:** Retrieves an identity node by its user ID.
- **get_neighbors:** Retrieves the neighboring identity nodes of a given user ID.
- **get_identity_nodes:** Retrieves identity nodes based on a specific identity type.
- **TryFrom<(String, String)> for IdentityType:** Converts a tuple of identity label and value into an IdentityType.

#### IdentityGraphHandler
- **new:** Creates a new instance of IdentityGraphHandler with the given Neo4j client.
- **add_identity:** Adds or updates an identity in the graph database.
- **get_user_identities:** Retrieves identities for a specific user from the graph database.

### Methods

#### IdentityType
- **as_tuple:** Returns the identity type as a tuple of its label and value.

#### IdentityGraph
- **new:** Creates a new instance of IdentityGraph.
- **add_node:** Adds a new identity node to the graph.
- **add_edge:** Adds a new identity edge to the graph.
- **get_node:** Retrieves an identity node by its user ID.
- **get_neighbors:** Retrieves the neighboring identity nodes of a given user ID.
- **get_identity_nodes:** Retrieves identity nodes based on a specific identity type.

#### IdentityGraphHandler
- **new:** Creates a new instance of IdentityGraphHandler with the given Neo4j client.
- **add_identity:** Adds or updates an identity in the graph database.
- **get_user_identities:** Retrieves identities for a specific user from the graph database.

### Dependencies
- **neo4rs:** Used for interacting with the Neo4j graph database.
- **std::sync::Arc:** Used for thread-safe reference counting.
- **std::convert::TryFrom:** Used for converting between types.
- **thiserror::Error:** Used for defining custom error types.
- **std::collections::HashMap:** Used for storing key-value pairs.

### Tests
- **test_identity_graph_handler:** Tests the functionality of the IdentityGraphHandler by adding identities and retrieving user identities from the graph database.

### Personalisation Graph
- **Path:** src/graphs/personalisation_graph.rs
- **Description:** This module builds a personalisation graph from text. It defines HashMap and HashSet data structures, performs NLP tasks such as tokenization, part-of-speech tagging, and dependency parsing. The `build_personalisation_graph_with_transfer_learning` function takes a string as input and returns a `Result` object that contains a `PersonalisationGraph` representing the personalisation graph.

#### Enums
- **PersonalisationNodeType:** Represents different types of nodes in the personalisation graph.
  - **Intent(String)**
  - **Entity(String)**
  - **Preference(String)**

#### Structs
- **PersonalisationNode:** Represents a node in the personalisation graph.
  - **node_type:** `PersonalisationNodeType`
  - **values:** `HashSet<String>`
- **PersonalisationGraph:** Represents a personalisation graph.
  - **nodes:** `HashMap<String, PersonalisationNode>`

#### Implementations
- **PersonalisationGraph**
  - **new:** Creates a new empty personalisation graph.
  - **add_node:** Adds a new node to the personalisation graph.
  - **get_node:** Gets a reference to a node in the personalisation graph.
  - **merge:** Merges another personalisation graph into this one.

#### Functions
- **build_personalisation_graph_with_transfer_learning:** Builds a personalisation graph from text using transfer learning.
  - **Input:** `text: &str, model_path: &str`
  - **Output:** `Result<PersonalisationGraph, Box<dyn std::error::Error>>`
  - The function performs the following steps:
    1. Initializes a dependency parser.
    2. Tokenizes and tags the input text.
    3. Parses dependencies to obtain dependency triples.
    4. Builds an initial personalisation graph based on the dependency triples.
    5. Extracts personalisation tokens from the knowledge graph using a trained Rasa NLU model.
    6. Adds the personalisation tokens to the knowledge graph.
    7. Initializes and trains a TED policy using the personalisation graph.
    8. Uses the TED policy to make predictions on the input text.
    9. Returns the predicted intent and entities.

- **main:** The main function that demonstrates the usage of the `build_personalisation_graph_with_transfer_learning` function.
  - Defines a string of conversational text.
  - Calls the `build_personalisation_graph_with_transfer_learning` function to build the personalisation graph, train the TED policy, and make predictions based on the input text.
  - Prints the predicted intent and entities to the console.

# Provider Graph Documentation

## Modules

### Provider Graph
- **Path:** src/graphs/provider_graph.rs
- **Description:** Defines structures and functionalities for managing service providers and their capabilities in a graph-like structure.

#### Structs
- **Provider:** Represents a service provider with a name and a set of capabilities.
  - **name:** The name of the provider.
  - **capabilities:** A HashMap of capabilities offered by the provider, keyed by their names.
- **Capability:** Represents a specific functionality offered by a provider, including the endpoints to access it and any required parameters.
  - **name:** The name of the capability.
  - **endpoints:** A vector of endpoint URLs for accessing the capability.
  - **parameters:** A HashMap of required parameters for the capability, keyed by their names.
- **Providers:** A registry for managing multiple providers.
  - **providers:** A HashMap of providers, keyed by their names.

#### Implementations
- **Provider:**
  - **new:** Creates a new provider with the given name and capabilities.
  - **fmt::Debug:** Implements the Debug trait for pretty-printing Provider instances.
- **Capability:**
  - **new:** Creates a new capability with the given name, endpoints, and parameters.
  - **fmt::Debug:** Implements the Debug trait for pretty-printing Capability instances.
- **Providers:**
  - **new:** Constructs a new, empty registry of providers.
  - **register:** Registers a new provider in the registry.
  - **get:** Retrieves a provider by name, if it exists.
  - **register_capability:** Registers a capability under a given provider, returning an error if the provider does not exist.
  - **get_capability:** Retrieves a specific capability of a provider by names, if both exist.
  - **remove_provider:** Removes a provider from the registry, returning the removed provider if it existed.
  - **remove_capability:** Removes a specific capability from a provider, returning the removed capability if it existed.
  - **list_capabilities:** Lists all capabilities of a given provider, if it exists.
  - **list_providers:** Lists all providers in the registry.

#### Functions
- **create_dynamic_provider:** Helper function to create a new provider with no capabilities.
- **create_dynamic_capability:** Helper function to create a new capability with specified endpoints and parameters.


# Schedule Graph Documentation

## Modules

### Schedule Graph
- **Path:** src/graphs/schedule_graph.rs
- **Description:** Defines structs and methods for managing events and dependencies in an itinerary. It provides a comprehensive framework for scheduling, modifying, and analyzing events based on their start and end times, resources, and dependencies on other events.

#### Structs
- **Event:** Represents a single event with a start time, end time, and a resource identifier. It forms the basic unit of scheduling within this system.

#### Implementations
- **Ord for Event:** Implements the Ord trait for Event, allowing events to be compared based on their start time.
- **PartialOrd for Event:** Implements the PartialOrd trait for Event, enabling partial ordering of events based on their start time.
- **Dependency:** Represents an event along with a vector of other events it depends on. This structure allows for the representation of complex relationships and constraints between various events in the itinerary.
- **Itinerary:** Contains a vector of Dependency structs, each encapsulating an Event and its dependencies, alongside a hash map (HashMap) of schedules for each resource. This struct is central to managing the collection of events and their scheduling.

#### Implementations
- **Itinerary:**
  - **new:** Creates a new Itinerary instance from a vector of Dependency structs.
  - **insert_event:** Inserts a new event (Dependency) into the itinerary and reschedules the events.
  - **modify_event:** Modifies an existing event in the itinerary at the specified index and reschedules the events.
  - **remove_event:** Removes an event from the itinerary at the specified index and reschedules the events.
  - **reschedule:** Reschedules the events in the itinerary based on their start times and updates the resource schedules.
  - **print_schedules:** Prints the schedules for each resource in the itinerary.
  - **find_events_by_resource:** Finds all events associated with a specific resource in the itinerary.
  - **find_overlapping_events:** Finds pairs of events that have overlapping time ranges in the itinerary.
  - **calculate_total_duration:** Calculates the total duration of all events in the itinerary.
  - **find_free_time_slots:** Finds free time slots for a specific resource in the itinerary, given a minimum duration.
  - **find_events_in_range:** Finds events that fall within a specific time range in the itinerary.
  - **find_dependent_events:** Finds events that depend on a specific event in the itinerary.
  - **calculate_critical_path:** Calculates the critical path (longest path) in the itinerary.

#### Functions
- **schedule_resources:** Schedules events based on their dependencies and resource availability, returning a HashMap of resource schedules.
- **generate_schedules:** Generates sample schedules for demonstration purposes.

#### Dependencies
- **std::collections::HashMap:** Used for storing resource schedules.
- **std::cmp::Ordering:** Used for comparing events based on their start time.

# Spatial Events Graph Documentation

## Modules

### Spatial Events Graph
- **Path:** src/graphs/spatial_events_graph.rs
- **Description:** Defines structs and methods for managing events and alerts in a spatial context.

#### Structs
- **Event:** Represents an event with a location, significance, and tags.
  - `location`: A tuple of three floats representing the event's location.
  - `significance`: A float representing the event's significance.
  - `tags`: A vector of strings representing the tags associated with the event.
- **Alert:** Represents an alert message for an event.
  - `event_name`: A string representing the name of the event.
  - `message`: A string representing the alert message.
  - `severity`: A float representing the severity of the alert.
- **EventGraph:** Represents a graph of events and alerts.
  - `events`: A HashMap of events keyed by their name.
  - `alerts`: A Vec of alerts for events.

#### Implementations
- **EventGraph:**
  - **new:** Creates a new instance of the EventGraph struct with an empty event list and alert list.
  - **add_event:** Adds a new event to the graph with the given name, location, significance, and tags.
  - **get_weighted_graph:** Calculates the weight of each event based on the agent's preferences, which are provided as a HashMap of event names to preference values.
  - **get_nearby_events:** Finds all events within a certain distance of a given location.
  - **add_alerts_along_path:** Adds alerts to the alerts vector in self if there are any events along a given path.
  - **generate_alert:** Generates an alert for a given event if it is in the weighted graph.
  - **calculate_significance_stats:** Calculates the mean and standard deviation of event significance.
  - **calculate_distance:** Calculates the distance between two locations.
  - **calculate_alert_severity:** Calculates the severity of an alert based on the weight of the event and the maximum weight of all events.
  - **filter_events_by_tags:** Filters events based on a given set of tags and returns a vector of events that have at least one of the specified tags.
  - **get_events_sorted_by_significance:** Returns a vector of events sorted by their significance in descending order.
  - **get_top_events_by_weight:** Returns the top N events based on their weights calculated using the provided preferences.

#### Methods
- **EventGraph::new:** Creates a new instance of the EventGraph struct with an empty event list and alert list.
- **EventGraph::add_event:** Adds a new event to the graph with the given name, location, significance, and tags.
- **EventGraph::get_weighted_graph:** Calculates the weight of each event based on the agent's preferences, which are provided as a HashMap of event names to preference values.
- **EventGraph::get_nearby_events:** Finds all events within a certain distance of a given location.
- **EventGraph::add_alerts_along_path:** Adds alerts to the alerts vector in self if there are any events along a given path.
- **EventGraph::generate_alert:** Generates an alert for a given event if it is in the weighted graph.
- **EventGraph::calculate_significance_stats:** Calculates the mean and standard deviation of event significance.
- **EventGraph::calculate_distance:** Calculates the distance between two locations.
- **EventGraph::calculate_alert_severity:** Calculates the severity of an alert based on the weight of the event and the maximum weight of all events.
- **EventGraph::filter_events_by_tags:** Filters events based on a given set of tags and returns a vector of events that have at least one of the specified tags.
- **EventGraph::get_events_sorted_by_significance:** Returns a vector of events sorted by their significance in descending order.
- **EventGraph::get_top_events_by_weight:** Returns the top N events based on their weights calculated using the provided preferences.

#### Dependencies
- **std::collections::HashMap:** Used for storing events and preferences.

# User Graph Documentation

## User Graph
- **Path:** src/graphs/user_graph.rs
- **Description:** Defines the UserGraph struct and its associated methods for representing and analyzing a graph of users, groups, and their interactions.

### Structs
- **UserGraph:** Represents the user graph, containing nodes, users, and groups.
- **Node:** Represents a node in the user graph, containing messages, user or group ID, reward, and edges.
- **Message:** Represents a message in a node, containing text, feedback weights, outgoing edges, and sentiment analysis.
- **Edge:** Represents an edge between nodes, containing weight, target node index, and reward.

### Implementations
#### UserGraph:
- **from_file:** Constructs a UserGraph from a JSON file.
- **analyze_sentiment:** Analyzes the sentiment of each message in the user graph.
- **get_top_messages_by_feedback:** Returns the top messages based on their total feedback weight.
- **get_top_nodes_by_feedback:** Returns the top nodes based on the total feedback weight of their messages.
- **get_connected_nodes:** Returns the nodes connected to a given node.
- **get_strongly_connected_components:** Returns the strongly connected components of the user graph.
- **dfs:** Performs a depth-first search on the user graph.
- **dfs_reverse:** Performs a reverse depth-first search on the user graph.

### Functions
- **run_reinforcement_learning:** Runs the reinforcement learning algorithm on the user graph.
- **update_exploration_rate:** Updates the exploration rate based on the current iteration.
- **update_message_feedback:** Updates the feedback weights of a message based on the received feedback.
- **log_progress:** Logs the progress of the reinforcement learning algorithm.
- **calculate_total_reward:** Calculates the total reward of the user graph.
- **simulate_action:** Simulates an action in the user graph and returns the next state and reward.
- **read_message:** Reads a message from a node based on the given action.
- **process_feedback:** Processes the feedback text and converts it into a vector of feedback weights.

### Dependencies
- **serde:** Used for serializing and deserializing the user graph and its components.
- **sentiment:** Used for sentiment analysis of messages.
- **std::fs::File:** Used for reading the user graph from a file.
- **std::io::{BufRead, BufReader}:** Used for buffered reading of the user graph file.
- **std::path::Path:** Used for handling file paths.

### Constants
- **INITIAL_EXPLORATION_RATE:** The initial exploration rate for the reinforcement learning algorithm.
- **MIN_EXPLORATION_RATE:** The minimum exploration rate for the reinforcement learning algorithm.
- **MAX_ITERATIONS:** The maximum number of iterations for the reinforcement learning algorithm.
- **EARLY_STOPPING_THRESHOLD:** The threshold for early stopping of the reinforcement learning algorithm based on improvement.

### Methods
- **QLearningAgent::reset_state:** Resets the state of the Q-learning agent to the starting state.

# Reinforcement Learning with Human Feedback (RLHF)

## Path
- src/recommendations/rlhf.rs

## Description
- Implements a reinforcement learning with human feedback (RLHF) algorithm for optimizing decision-making in a user graph.

## Constants
- **INITIAL_EXPLORATION_RATE:** The initial exploration rate for the Q-learning agent.
- **MIN_EXPLORATION_RATE:** The minimum exploration rate for the Q-learning agent.
- **MAX_ITERATIONS:** The maximum number of iterations for the learning process.
- **EARLY_STOPPING_THRESHOLD:** The threshold for early stopping based on improvement in total rewards.

## Structs
- **RLHFConfig:** Represents the configuration parameters for the RLHF algorithm.

## Functions
- **run_reinforcement_learning:** Executes the reinforcement learning process on a user graph using the Q-learning algorithm.
- **update_exploration_rate:** Updates the exploration rate based on the current iteration and configuration.
- **update_message_feedback:** Updates the feedback weights of a message based on the received feedback.
- **log_progress:** Logs the progress of the learning process, including iteration count, total rewards, and exploration rate.
- **calculate_total_reward:** Calculates the total reward of the user graph.
- **simulate_action:** Simulates an action in the user graph and returns the next state and reward.
- **read_message:** Reads the message associated with a given action in the user graph.
- **process_feedback:** Processes the feedback text and converts it into a vector of feedback weights.

## Implementations
- **Default for RLHFConfig:** Provides default values for the RLHF configuration parameters.

### QLearningAgent
- **reset_state:** Resets the agent's state to the starting state.
- **save_q_table:** Saves the Q-table to a file.

## Methods
### run_reinforcement_learning
- Initializes a Q-learning agent with the user graph dimensions and learning parameters.
- Executes a learning loop where the agent:
  - Observes the current state and selects an action based on an epsilon-greedy strategy.
  - Simulates the chosen action within the user graph context and acquires feedback.
  - Processes the feedback to update the feedback weights for the current action's message.
  - Updates the Q-value for the observed state-action pair, incorporating the received reward and anticipated future rewards.
  - Progresses to the next state, with the capability to reset the agent's state upon reaching a terminal condition.
  - Adapts the exploration rate over time to balance exploration and exploitation.
  - Implements an early stopping mechanism based on the stabilization of the learning process.
  - Logs the learning progress metrics for monitoring and analysis purposes.
  - Saves the learned Q-table to a file.

## Dependencies
- **crate::graphs::user_graph:** Provides the UserGraph structure for representing the user graph.
- **crate::agents::q_learning_agent:** Provides the QLearningAgent for executing the Q-learning algorithm.
- **std::io:** Used for handling I/O errors.
- **serde:** Used for serializing and deserializing the RLHF configuration and Q-table.


# Group Module Documentation

## Modules

### Group
- **Path:** src/iam/group.rs
- **Description:** Defines the Group struct and associated functionality for managing groups of users.

#### Structs
- **Group:** Represents a group of users with various fields:
    - **id:** The unique identifier of the group (String).
    - **name:** The name of the group (String).
    - **description:** An optional description of the group (Option<String>).
    - **users:** The list of users in the group (Vec<User>).
    - **created_at:** The timestamp when the group was created (DateTime).
    - **last_modified_at:** The timestamp when the group was last modified (DateTime).
    - **max_users:** The maximum number of users allowed in the group (Option<usize>).

#### Enums
- **GroupError:** Represents errors that can occur when interacting with a group:
    - **UserNotFound:** Indicates that a user was not found in the group.
    - **UserAlreadyExists:** Indicates that a user already exists in the group.
    - **GroupFull:** Indicates that the group has reached its maximum capacity.

#### Implementations
- **Group:**
    - **new:** Creates a new group with the given id, name, and max_users.
    - **set_description:** Sets the description of the group.
    - **add_user:** Adds a user to the group. Returns an error if the user already exists or the group is full.
    - **remove_user:** Removes a user from the group based on the user's id. Returns the removed user if found, otherwise returns an error.
    - **find_user:** Finds a user in the group based on the user's id. Returns a reference to the user if found, otherwise returns None.
- **fmt::Display for Group:** Implements the Display trait for the Group struct, allowing it to be formatted as a string.

#### Methods
- **Group::new:** Creates a new group with the given id, name, and max_users.
- **Group::set_description:** Sets the description of the group.
- **Group::add_user:** Adds a user to the group. Returns an error if the user already exists or the group is full.
- **Group::remove_user:** Removes a user from the group based on the user's id. Returns the removed user if found, otherwise returns an error.
- **Group::find_user:** Finds a user in the group based on the user's id. Returns a reference to the user if found, otherwise returns None.

#### Dependencies
- **chrono:** Used for handling timestamps (DateTime).
- **serde:** Used for serializing and deserializing the Group struct.
- **thiserror:** Used for defining custom error types (GroupError).
- **std::fmt:** Used for implementing the Display trait for the Group struct.

# Identity and Access Management (IAM)

## Path: src/iam/iam.rs
- **Description:** Implements identity and access management functionality using Keycloak as the backend provider. Provides methods for user management, authentication, and authorization.

### Structs
- **Token:** Represents an access token returned by Keycloak, containing various token-related fields.
- **KeycloakUserManager:** Manages Keycloak user operations, such as filtering, creating, updating, and deleting users.
- **KeycloakUserModel:** Represents a Keycloak user model with fields like username and email.
- **KeycloakController:** Provides high-level methods for interacting with Keycloak, including user management, authentication, and authorization.

### Enums
- **KeycloakError:** Defines custom error types for various Keycloak operations, such as realm creation, user creation, authentication, and user management errors.

### Implementations
#### KeycloakUserManager
- **new:** Creates a new instance of KeycloakUserManager with the given KeycloakController.
- **filter:** Filters users based on a specific field and value.
- **create_user:** Creates a new user in Keycloak using the provided KeycloakUserModel.
- **update_user:** Updates an existing user in Keycloak using the provided KeycloakUserModel.
- **delete_user:** Deletes a user from Keycloak based on the provided username.
- **create_user_wallet:** Creates a new wallet for a user and stores the wallet information in Keycloak user attributes.
- **get_user_wallet:** Retrieves a user's wallet information from Keycloak user attributes.
- **issue_credential:** Issues a verifiable credential to a user by signing it with the user's wallet.
- **verify_credential:** Verifies a verifiable credential using the issuer's wallet.

#### KeycloakUserModel
- **to_user_representation:** Converts a KeycloakUserModel to a UserRepresentation used by the Keycloak API.

#### KeycloakController
- **issue_credential:** Issues a verifiable credential to a user using the KeycloakUserManager.
- **verify_credential:** Verifies a verifiable credential using the KeycloakUserManager.
- **create_user_wallet:** Creates a new wallet for a user using the KeycloakUserManager.
- **get_user_wallet:** Retrieves a user's wallet using the KeycloakUserManager.

### Functions
- **main:** The main function demonstrates the usage of various Keycloak operations, such as creating a realm, creating a user, generating an OpenID token, authenticating a user, logging out a user, and performing user management operations like filtering, creating, updating, and deleting users.

### Dependencies
- **base64:** Used for encoding and decoding base64 strings.
- **keycloak_provider:** Provides Keycloak-related functionality, such as creating realms, creating users, and managing user representations.
- **reqwest:** Used for making HTTP requests to the Keycloak API.
- **tokio:** Used for asynchronous programming and running the main function.
- **serde:** Used for serializing and deserializing data structures.
- **std::time:** Used for handling timestamps.
- **thiserror:** Used for defining custom error types.
- **crate::jwt:** Used for signing and verifying verifiable credentials using wallets.

# JWT (JSON Web Token) Module

## Path: src/iam/jwt.rs
- **Description:** Implements JSON Web Token (JWT) functionality for authentication and authorization.

### Structs
- **JWT:** Represents a JSON Web Token, containing a header, payload, and optional signature.
- **Jwks:** Represents a set of JSON Web Keys (JWKs).
- **Jwk:** Represents a single JSON Web Key (JWK).
- **JwksCenter:** Manages the retrieval and selection of JWKs from a remote endpoint.
- **JWKSEndpoint:** Represents an endpoint for managing JWKs stored in a PostgreSQL database.
- **Claims:** Represents the claims contained within a JWT.

### Implementations
#### JWT
- **empty:** Creates a new empty JWT with a default header and empty payload.
- **add_payload:** Adds a key-value pair to the JWT payload.
- **get_payloads:** Retrieves the entire payload of the JWT.
- **get_payload:** Retrieves a specific value from the JWT payload by key.
- **encode:** Encodes the JWT using a randomly selected JWK from the JwksCenter.
- **decode:** Decodes a JWT string into a JWT struct, verifying the signature using the appropriate JWK.

#### Jwk
- **kid:** Retrieves the key ID (kid) of the JWK.
- **pem:** Retrieves the PEM-encoded key material of the JWK.

#### JwksCenter
- **new:** Creates a new instance of JwksCenter with the given URL.
- **select_random:** Selects a random JWK from the set of JWKs retrieved from the remote endpoint.
- **query:** Retrieves a specific JWK by its key ID (kid) from the set of JWKs retrieved from the remote endpoint.
- **query_from_url:** Retrieves the set of JWKs from the remote endpoint.

#### JWKSEndpoint
- **new:** Creates a new instance of JWKSEndpoint with the given PostgreSQL client.
- **list:** Retrieves all JWKs stored in the PostgreSQL database.
- **add:** Adds a new JWK to the PostgreSQL database.

### Functions
- **encode_token:** Encodes a set of claims into a JWT string using the provided secret.
- **decode_token:** Decodes a JWT string into a set of claims using the provided secret.
- **sign_credential_with_wallet:** Signs a verifiable credential using a wallet's signing key and returns the signed credential as a JSON string.
- **verify_credential_with_wallet:** Verifies a signed verifiable credential using a wallet's verification method.

### Constants
- **JWK_CENTER_URL:** The URL of the remote endpoint for retrieving JWKs.

### Dependencies
- **jsonwebtoken:** Used for encoding and decoding JWTs.
- **reqwest:** Used for making HTTP requests to the remote JWK endpoint.
- **serde:** Used for serializing and deserializing JSON data.
- **chrono:** Used for handling dates and times.
- **ockam:** Used for generating random values.
- **crate::clients::kv::KVStore:** Used for interacting with a key-value store.
- **crate::clients::postgres::PGTableKVClient:** Used for interacting with a PostgreSQL database.
- **crate::BigbotError:** Custom error type used throughout the module.
- **crate::credentials::VerifiableCredential:** Represents a verifiable credential.
- **crate::wallet::Wallet:** Represents a wallet used for signing and verifying credentials.


# Keycloak Provider Documentation

## Keycloak Provider
- **Path:** src/iam/keycloak_provider.rs
- **Description:** Provides functionality for interacting with a Keycloak server to create realms and users.

### Structs
- **RealmCreationRequest:** Represents the request payload for creating a new realm in Keycloak.
  - **id:** The ID of the realm to be created.
  - **realm:** The name of the realm to be created.
- **UserCreationRequest:** Represents the request payload for creating a new user in Keycloak.
  - **username:** The username of the user to be created.
  - **enabled:** Indicates whether the user should be enabled or not.
- **UserRepresentation:** Represents the response payload received after creating a user in Keycloak.
  - **id:** The ID of the created user.
  - **username:** The username of the created user.

### Functions
- **create_realm:**
  - **Description:** Creates a new realm in Keycloak.
  - **Parameters:**
    - **client:** A reference to the reqwest::Client used for making HTTP requests.
    - **admin_token:** The bearer token for authenticating with the Keycloak server.
    - **realm_name:** The name of the realm to be created.
  - **Returns:** A Result indicating the success or failure of the realm creation operation.
- **create_user:**
  - **Description:** Creates a new user in a specific realm in Keycloak.
  - **Parameters:**
    - **client:** A reference to the reqwest::Client used for making HTTP requests.
    - **admin_token:** The bearer token for authenticating with the Keycloak server.
    - **realm_name:** The name of the realm in which the user should be created.
    - **username:** The username of the user to be created.
  - **Returns:** A Result containing the UserRepresentation of the created user, or an error if the operation fails.

### Dependencies
- **reqwest:** Used for making HTTP requests to the Keycloak server.
- **serde:** Used for serializing and deserializing request and response payloads.
- **jwt_utils:** Assumed to be in scope, used for encoding and decoding JWT tokens.
- **utils:** Assumed to be in scope, used for generating random alphanumeric strings.

### Usage
The **create_realm** function can be used to create a new realm in Keycloak by providing the necessary parameters such as the reqwest::Client, admin token, and realm name. It sends a POST request to the Keycloak server with the realm creation request payload and returns a Result indicating the success or failure of the operation.

The **create_user** function can be used to create a new user in a specific realm in Keycloak. It requires the reqwest::Client, admin token, realm name, and username as parameters. It sends a POST request to the Keycloak server with the user creation request payload and returns a Result containing the UserRepresentation of the created user, or an error if the operation fails.

# Merkle Tree Implementation

## Path
src/iam/merkle_tree.rs

## Description
Implements a Merkle tree data structure for efficient verification of membership and generation of proofs.

### Structs
- **CompressedMerkleProof:** Represents a compressed Merkle proof, containing the path, hashes, and leaf index.
- **MerkleTree:** Represents a Merkle tree, storing the tree levels, next index, root hash, filled subtrees, zero values, and node hashes.

### Implementations
#### CompressedMerkleProof
- **new:** Creates a new instance of CompressedMerkleProof with the given path, hashes, and leaf index.
- **verify:** Verifies the Merkle proof against the provided root hash and leaf hash.

#### MerkleTree
- **new:** Creates a new instance of MerkleTree with default values.
- **update:** Updates the Merkle tree with a new leaf, returning the leaf index.
- **get_proof:** Retrieves the Merkle proof for a given leaf index.
- **sibling_path:** Computes the sibling path for a given path.
- **hash_combine:** Combines two hash values using the Keccak256 hash function.
- **arrays_equal:** Checks if two Uint8Arrays are equal.

### Methods
#### CompressedMerkleProof
- **new:** Creates a new instance of CompressedMerkleProof with the given path, hashes, and leaf index.
- **verify:** Verifies the Merkle proof against the provided root hash and leaf hash. It iteratively combines the leaf hash with the sibling hashes based on the path and compares the final hash with the root hash.

#### MerkleTree
- **new:** Creates a new instance of MerkleTree with default values for levels, next index, root hash, filled subtrees, zero values, and node hashes.
- **update:** Updates the Merkle tree with a new leaf, returning the leaf index. It computes the path and hashes for the new leaf, updates the node hashes, and increments the next index.
- **get_proof:** Retrieves the Merkle proof for a given leaf index. It computes the path from the leaf to the root and collects the sibling hashes along the way, returning a CompressedMerkleProof.
- **sibling_path:** Computes the sibling path for a given path by flipping the last bit of the path.
- **hash_combine:** Combines two hash values using the Keccak256 hash function. It concatenates the left and right hashes and computes the Keccak256 hash of the combined data.
- **arrays_equal:** Checks if two Uint8Arrays are equal by comparing their lengths and element values.

### Dependencies
- **js_sys:** Provides bindings for JavaScript types, such as Uint8Array.
- **wasm_bindgen:** Enables interoperability between Rust and JavaScript in WebAssembly.
- **std::collections::HashMap:** Used for storing node hashes in the Merkle tree.
- **keccak256:** Used for computing the Keccak256 hash function.


# User Management

## Path: src/iam/user.rs
- **Description:** Provides a comprehensive solution for managing user entities within a system, including functionality for creating, updating, deleting, and retrieving user records, along with managing their associated data like wallets and verifiable credentials.

### Structs
- **User:** Represents a user in the system, containing fields for id, username, email, wallet, credentials, credential_tree, created_at, and last_modified_at.
- **UserBuilder:** Represents a builder for creating a user, allowing for a fluent interface when constructing a new user instance.
- **UserService:** Represents a service for managing users, providing methods for creating, retrieving, updating, and deleting users.

### Enums
- **UserError:** Represents errors that can occur when interacting with a user, including UserNotFound, UserCreationError, UserUpdateError, and UserDeleteError.

### Implementations
#### User
- **new:** Creates a new user with the given id, username, email, and wallet.
- **set_username:** Updates the user's username.
- **set_email:** Updates the user's email address.
- **set_wallet:** Updates the user's wallet.
- **add_credential:** Adds a new verifiable credential to the user.
- **get_credential:** Retrieves a verifiable credential by its ID.
- **generate_credential_proof:** Generates a proof for a specific credential.
- **Display:** Implements the Display trait for User, providing a formatted string representation of the user.

#### UserBuilder
- **new:** Creates a new user builder.
- **username:** Sets the username for the user.
- **email:** Sets the email address for the user.
- **wallet:** Sets the wallet for the user.
- **build:** Builds and returns the user.

#### UserService
- **new:** Creates a new user service.
- **create_user:** Creates a new user with the given details.
- **get_user:** Retrieves a user by their ID.
- **update_user:** Updates an existing user.
- **delete_user:** Deletes a user by their ID.

### Methods
- **User::new:** Creates a new user with the given id, username, email, and wallet.
- **User::set_username:** Updates the user's username.
- **User::set_email:** Updates the user's email address.
- **User::set_wallet:** Updates the user's wallet.
- **User::add_credential:** Adds a new verifiable credential to the user.
- **User::get_credential:** Retrieves a verifiable credential by its ID.
- **User::generate_credential_proof:** Generates a proof for a specific credential.
- **UserBuilder::new:** Creates a new user builder.
- **UserBuilder::username:** Sets the username for the user.
- **UserBuilder::email:** Sets the email address for the user.
- **UserBuilder::wallet:** Sets the wallet for the user.
- **UserBuilder::build:** Builds and returns the user.
- **UserService::new:** Creates a new user service.
- **UserService::create_user:** Creates a new user with the given details.
- **UserService::get_user:** Retrieves a user by their ID.
- **UserService::update_user:** Updates an existing user.
- **UserService::delete_user:** Deletes a user by their ID.

### Dependencies
- **chrono:** Used for handling timestamps (DateTime).
- **serde:** Used for serializing and deserializing user data.
- **std::collections::HashMap:** Used for storing user credentials.
- **thiserror:** Used for defining custom error types (UserError).
- **std::fmt:** Used for implementing the Display trait for User.
- **crate::iam::proofs::MerkleTree:** Used for managing the user's credential proofs.
- **crate::iam::verifiable_credentials::{VerifiableCredential, VCBuilder}:** Used for handling verifiable credentials.
- **crate::iam::wallet::Wallet:** Used for managing the user's wallet.

# Verifiable Credentials Documentation

## Modules

### Verifiable Credentials
- **Path:** src/iam/verifiable_credentials.rs
- **Description:** Implements functionality for working with verifiable credentials, including creation, verification, and interaction with web3 wallets.

#### Structs
- **VerifiableCredential:** Represents a verifiable credential, containing context, types, ID, issuer, issuance date, proof, and credential subject.
- **CredentialSubject:** Represents the subject of a verifiable credential, containing an ID and a wallet address.
- **VCBuilder:** A builder struct for creating VerifiableCredential instances.
- **Proof:** Represents the proof of a verifiable credential, containing proof type, creation date, verification method, and JWT.
- **Jwks:** Represents a set of JSON Web Keys (JWKs), containing a vector of Jwk instances.
- **Jwk:** Represents a single JSON Web Key (JWK), containing a key ID and a PEM-encoded public key.

#### Enums
- **Degree:** Represents different types of academic degrees (e.g., BachelorOfArts, MasterOfScience, etc.).

#### Implementations
##### VCBuilder
- **default:** Creates a new instance of VCBuilder with default values.
- **add_context:** Adds a context to the verifiable credential.
- **add_type:** Adds a type to the verifiable credential.
- **set_id:** Sets the ID of the verifiable credential.
- **set_issuer:** Sets the issuer of the verifiable credential.
- **set_issuance_date:** Sets the issuance date of the verifiable credential.
- **set_proof:** Sets the proof of the verifiable credential.
- **set_subject_id:** Sets the ID of the credential subject.
- **set_subject_wallet_address:** Sets the wallet address of the credential subject.
- **build:** Builds and returns the VerifiableCredential instance.

##### VerifiableCredential
- **get_proof:** Returns an optional reference to the proof of the verifiable credential.
- **get_subject_wallet_address:** Returns the wallet address of the credential subject.

##### Proof
- **new:** Creates a new instance of Proof with the given JWT.
- **jwt:** Returns an optional reference to the JWT string.

#### Functions
- **verify_credential:** Verifies a verifiable credential by checking the JWT signature and claims against the provided issuer and JWKS URI.
- **decode_header:** Decodes the header of a JWT and returns a jsonwebtoken::Header instance.
- **sign_credential_with_wallet:** Signs a verifiable credential using a web3 wallet and returns the signed credential as a JSON string.
- **verify_credential_with_wallet:** Verifies a verifiable credential using a web3 wallet and returns a boolean indicating the verification result.

#### Dependencies
- **chrono:** Used for working with dates and times.
- **jsonwebtoken:** Used for decoding and validating JWTs.
- **reqwest:** Used for making HTTP requests to download JWKs.
- **serde:** Used for serializing and deserializing JSON data.
- **serde_json:** Used for working with JSON values and strings.
- **web3:** Used for working with Ethereum, Solana and Polkadot addresses.

# Message Classifier Documentation

## Module

### Message Classifier
- **Path:** src/messaging/message_classifier.rs
- **Description:** Provides functionality to classify messages based on their metadata and entity graph.

#### Functions
- **classify_message:**
  - **Description:** Classifies a message based on its metadata and entity graph.
  - **Parameters:**
    - `metadata: &HashMap<String, MetadataValue>`: A reference to a HashMap containing the message metadata, where the keys are metadata field names and the values are MetadataValue enums.
    - `entity_graph: &EntityGraph`: A reference to an EntityGraph representing the entities present in the message.
  - **Return Value:** A String representing the classified message type.
  - **Classification Logic:**
    - Initializes an empty classification string.
    - Checks if the `entity_graph` contains any entities of type Location. If so, sets the classification to "Location-based message".
    - If there are no Location entities, checks if the metadata HashMap has a "reply_to" key with a MetadataValue of type ReplyInfo. If so, sets the classification to "Reply message".
    - If the message is not a reply, checks if the metadata HashMap has a "media" key with a MetadataValue of type MediaAttachment. If so, sets the classification to "Media message".
    - If the message is not a media message, checks if the metadata HashMap has a "post" key with a MetadataValue of type Bool(true). If so, sets the classification to "Post message".
    - If the message is not a post, checks if the metadata HashMap has a "pinned" key with a MetadataValue of type Bool(true). If so, sets the classification to "Pinned message".
    - If none of the above conditions are met, sets the classification to "Regular message".
    - Returns the classified message type as a String.

#### Dependencies
- **entity_graph:** Provides the EntityGraphTrait and EntityType for working with entity graphs.
- **message_metadata:** Provides the MetadataValue enum for representing message metadata values.
- **std::collections::HashMap:** Used for storing and accessing message metadata as key-value pairs.


# Message Encryption

## Path: src/messaging/message_encryption.rs
- **Description:** This module orchestrates end-to-end encrypted messaging using the Ockam framework, leveraging MongoDB for message storage, Kafka for message routing, UDP for message transport, and secure channels for encrypting communications between senders and recipients.

### Structs
- **Message:** Represents a message with sender, recipient, and body fields.
- **EncryptedMessageHandler:** A worker responsible for handling encrypted messages, including decryption, storage, re-encryption, and forwarding.
- **KeyExchangeHandler:** A worker responsible for handling key exchange requests, initiating key exchanges, and establishing shared secrets with remote nodes.

### Implementations
#### Worker for EncryptedMessageHandler:
- **handle_message:** Decrypts incoming messages, deserializes them into the Message struct, stores them in MongoDB, re-encrypts them for the intended recipient, and forwards them to Kafka for distribution.

#### Worker for KeyExchangeHandler:
- **handle_message:** Deserializes key exchange requests, initiates key exchanges to establish shared secrets with remote nodes, and sends key exchange responses via Kafka.

### Functions
- **run:** The main function that initializes the Ockam node, sets up connections to MongoDB and Kafka, activates UDP transport, establishes a secure channel, starts the EncryptedMessageHandler and KeyExchangeHandler workers, and runs the Ockam node.
- **main:** The entry point of the program, which calls the run function and handles any errors.

### Dependencies
- **ockam:** The core Ockam library for building secure, decentralized applications.
- **ockam_kafka:** Ockam's Kafka integration for message routing.
- **ockam_mongo:** Ockam's MongoDB integration for message storage.
- **ockam_node:** Ockam's node implementation for running the application.
- **ockam_transport:** Ockam's transport layer, including UDP support.
- **ockam_vault_sync_core:** Ockam's vault synchronization core, providing key exchange algorithms (XX) and key types (PublicKey, SecretKey).
- **serde:** A serialization/deserialization framework for converting between Rust data types and various data formats, such as JSON.
- **tokio:** An asynchronous runtime for Rust, used for running the Ockam node and handling concurrent tasks.

### Workflow
1. Initialize a new Ockam node, set up connections to MongoDB and Kafka, and activate UDP transport.
2. Establish a secure channel utilizing the XX New Key Exchanger to facilitate encrypted communications between the sender and recipient.
3. Deploy the EncryptedMessageHandler worker to decrypt incoming messages, deserialize them into the Message struct, store them in MongoDB, re-encrypt them for the intended recipient, and forward them to Kafka for distribution.
4. Initiate the KeyExchangeHandler worker to process key exchange requests by deserializing them, initiate and manage key exchanges to establish shared secrets with remote nodes, and respond to key exchange requests via Kafka.
5. Execute the Ockam node, generating a local node key pair, and commence the operation of both workers to manage encrypted message processing and key exchange requests efficiently.
6. Subscribe to a key exchange topic to manage incoming key exchange requests, ensuring the system is equipped to handle encrypted messages and support dynamic key exchange mechanisms.

# Message Metadata HashMap Documentation

## Modules

### Message Metadata HashMap
- **Path:** src/messaging/message_hashmap.rs
- **Description:** Defines a HashMap-based structure for storing and managing message metadata, along with related enums and structs.

#### Structs
- **ReplyInfo:** Represents reply information for a message.
- **MediaAttachment:** Represents a media attachment in a message.
- **MessageEntity:** Represents an entity in a message, such as a mention or hashtag.
- **Reaction:** Represents a reaction to a message.
- **MessageMetadata:** Represents the metadata of a message, containing a HashMap of MetadataValue instances.

#### Enums
- **MetadataValue:** Represents the possible values for metadata fields in a message. It is an enum with variants for different data types:
  - **Int:** Represents an integer value.
  - **String:** Represents a string value.
  - **Bool:** Represents a boolean value.
  - **OptionInt:** Represents an optional integer value.
  - **ReplyInfo:** Represents reply information for a message.
  - **MediaAttachment:** Represents a media attachment in a message.
  - **Entities:** Represents a vector of MessageEntity instances.
  - **Reactions:** Represents a vector of Reaction instances.

#### Functions
- **print_message_metadata:** Creates a HashMap of message metadata values, inserts various metadata fields into the HashMap, creates a MessageMetadata instance from the HashMap, and prints the MessageMetadata instance.

#### Dependencies
- **std::collections::HashMap:** Used for storing and managing the message metadata.
- **messagemetadata::{MessageMetadata, MetadataValue}:** Imports the MessageMetadata and MetadataValue types from the messagemetadata module.

# Message Metadata Documentation

## Modules

### Message Metadata
- **Path:** src/messaging/message_metadata.rs
- **Description:** Defines the `MessageMetadata` struct and related types for representing metadata associated with messages in the messaging system.

#### Structs
- **MessageMetadata:** Represents the metadata of a message, containing a `HashMap` of key-value pairs where the keys are strings and the values are of type `MetadataValue`.

#### Enums
- **MetadataValue:** An enum representing the possible types of values that can be stored in the `MessageMetadata`. It includes the following variants:
  - **Bool:** Represents a boolean value.
  - **String:** Represents a string value.
  - **Int:** Represents a 64-bit signed integer value.
  - **OptionInt:** Represents an optional 64-bit signed integer value.
  - **ReplyInfo:** Represents information about a reply, stored as a boxed `ReplyInfo` struct.
  - **MediaAttachment:** Represents a media attachment, stored as a boxed `MediaAttachment` struct.
  - **Entities:** Represents a vector of `MessageEntity` structs.
  - **Reactions:** Represents a vector of `Reaction` structs.

#### Implementations
- **MessageMetadata:**
  - **new:** Creates a new instance of `MessageMetadata` with some default metadata values. In the provided example, it sets the "id" key to an `Int` value of 12345.

#### Dependencies
- **std::collections::HashMap:** Used for storing the metadata key-value pairs in the `MessageMetadata` struct.

# Message Routing

## Path: src/messaging/message_routing.rs

## Description: Implements message routing functionality for BigBot, including message classification, entity extraction, and routing messages to appropriate handlers based on their content and metadata.

### Structs
- **MessageRouter:** Represents the message router, which manages message classification, entity extraction, and routing.
  - **entity_extractor:** An instance of the EntityExtractor used for extracting entities from messages.
  - **intent_classifier:** An instance of the IntentClassifier used for classifying the intent of messages.
  - **topic_mapper:** An instance of the TopicMapper used for mapping topics to message handlers.
  - **handler_registry:** A HashMap that maps message types to their corresponding handlers.

### Enums
- **MessageType:** Represents the different types of messages that can be routed.
  - **Command:** Represents a command message.
  - **Query:** Represents a query message.
  - **Event:** Represents an event message.
  - **Unknown:** Represents an unknown message type.

### Traits
- **MessageHandler:** Defines the interface for message handlers.
  - **handle_message:** Handles a message and returns a result indicating the outcome of the operation.

### Implementations
- **MessageRouter:**
  - **new:** Creates a new instance of MessageRouter with the given EntityExtractor, IntentClassifier, and TopicMapper.
  - **register_handler:** Registers a message handler for a specific message type.
  - **route_message:** Routes a message to the appropriate handler based on its type and content.
  - **classify_message:** Classifies a message into a specific MessageType based on its content and metadata.
  - **extract_entities:** Extracts relevant entities from a message using the EntityExtractor.

### Functions
- **parse_message_type:** Parses a string representation of a message type into the corresponding MessageType enum variant.

### Methods
- **MessageRouter::new:** Creates a new instance of MessageRouter with the given EntityExtractor, IntentClassifier, and TopicMapper.
- **MessageRouter::register_handler:** Registers a message handler for a specific message type.
- **MessageRouter::route_message:** Routes a message to the appropriate handler based on its type and content.
- **MessageRouter::classify_message:** Classifies a message into a specific MessageType based on its content and metadata.
- **MessageRouter::extract_entities:** Extracts relevant entities from a message using the EntityExtractor.

### Dependencies
- **entity_extraction:** Module containing the EntityExtractor used for extracting entities from messages.
- **intent_classification:** Module containing the IntentClassifier used for classifying the intent of messages.
- **topic_mapping:** Module containing the TopicMapper used for mapping topics to message handlers.
- **std::collections::HashMap:** Used for storing the mapping between message types and their corresponding handlers.
- **std::fmt:** Used for formatting error messages.
- **std::str::FromStr:** Used for parsing string representations of message types into MessageType enum variants.

# Multi-Modal Inputs Documentation

## Modules

### Multi-Modal Inputs
- **Path:** src/messaging/multi_modal_inputs.rs
- **Description:** Defines structures and methods for handling multi-modal inputs in the messaging system.

#### Structs
- **MultiModalInputHandler:** Represents a handler for processing multi-modal inputs.
  
  ##### Fields
  - **nlp_client:** An instance of the NLP client for processing natural language inputs.
  - **speech_client:** An instance of the speech recognition client for processing speech inputs.
  - **vision_client:** An instance of the computer vision client for processing image inputs.
  
  ##### Methods
  - **new:** Creates a new instance of MultiModalInputHandler with the specified NLP, speech, and vision clients.
  - **process_text_input:** Processes a text input by calling the NLP client's process_text method and returns the result.
  - **process_speech_input:** Processes a speech input by calling the speech recognition client's recognize_speech method and returns the recognized text.
  - **process_image_input:** Processes an image input by calling the computer vision client's analyze_image method and returns the image analysis result.
  - **process_multi_modal_input:** Processes a multi-modal input by determining the input type (text, speech, or image) and calling the corresponding processing method. Returns the processed result.

- **MultiModalInput:** Represents a multi-modal input, which can be either text, speech, or image.
  
  ##### Variants
  - **Text(String):** Represents a text input as a string.
  - **Speech(Vec<u8>):** Represents a speech input as a byte vector.
  - **Image(Vec<u8>):** Represents an image input as a byte vector.

- **MultiModalInputResult:** Represents the result of processing a multi-modal input.
  
  ##### Variants
  - **Text(String):** Represents the processed text result as a string.
  - **Image(ImageAnalysis):** Represents the processed image result as an ImageAnalysis struct.

#### Enums
- **MultiModalInputError:** Represents the possible errors that can occur during multi-modal input processing.
  
  ##### Variants
  - **UnsupportedInputType:** Indicates that the input type is not supported.
  - **NlpError(String):** Represents an error that occurred during NLP processing, containing the error message.
  - **SpeechError(String):** Represents an error that occurred during speech recognition, containing the error message.
  - **VisionError(String):** Represents an error that occurred during image analysis, containing the error message.

#### Dependencies
- **crate::nlp::nlp_client::NlpClient:** The NLP client for processing natural language inputs.
- **crate::speech::speech_client::SpeechClient:** The speech recognition client for processing speech inputs.
- **crate::vision::vision_client::VisionClient:** The computer vision client for processing image inputs.
- **crate::vision::image_analysis::ImageAnalysis:** The struct representing the result of image analysis.

# PII Handler Documentation

## Modules

### PII Handler
- **Path:** src/messaging/pii_handler.rs
- **Description:** Implements a handler for detecting and masking Personally Identifiable Information (PII) in messages.

#### Structs
- **PiiHandler:** Represents the PII handler, which is responsible for detecting and masking PII in messages.

#### Enums
- **PiiType:** Represents the different types of PII that can be detected, such as Name, Email, Phone, Address, etc.

#### Implementations
- **PiiHandler:**
  - **new:** Creates a new instance of PiiHandler with the specified configuration options.
  - **detect_pii:** Detects PII in the given message using regular expressions or machine learning models based on the configured detection method.
  - **mask_pii:** Masks the detected PII in the message using the specified masking strategy (e.g., redaction, tokenization, or format-preserving encryption).
  - **handle_message:** Processes the incoming message by detecting and masking PII based on the configured rules and policies.

#### Methods
- **PiiHandler::new:** Creates a new instance of PiiHandler with the specified configuration options, such as the PII detection method, masking strategy, and PII types to detect.
- **PiiHandler::detect_pii:** Detects PII in the given message using regular expressions or machine learning models based on the configured detection method. Returns a vector of detected PII items with their corresponding PiiType and location within the message.
- **PiiHandler::mask_pii:** Masks the detected PII in the message using the specified masking strategy. Supports redaction (replacing PII with asterisks or a placeholder), tokenization (replacing PII with a random token), and format-preserving encryption (encrypting PII while preserving its format).
- **PiiHandler::handle_message:** Processes the incoming message by detecting and masking PII based on the configured rules and policies. Applies the specified PII detection method and masking strategy to the message and returns the masked message.

#### Dependencies
- **regex:** Used for detecting PII using regular expressions.
- **serde:** Used for serializing and deserializing configuration options.
- **tokio:** Used for asynchronous message processing.
- **tracing:** Used for logging and tracing the PII detection and masking process.

#### Configuration Options
- **detection_method:** Specifies the PII detection method to use, such as "regex" for regular expressions or "ml" for machine learning models.
- **masking_strategy:** Specifies the masking strategy to apply to detected PII, such as "redact", "tokenize", or "fpe" (format-preserving encryption).
- **pii_types:** Specifies the types of PII to detect, such as Name, Email, Phone, Address, etc.
- **regex_patterns:** Specifies the regular expression patterns for detecting each type of PII when using the "regex" detection method.
- **ml_model_path:** Specifies the path to the trained machine learning model for detecting PII when using the "ml" detection method.

### Message Router and Classifier
- **Path:** src/messaging/route_classifier.rs
- **Description:** Implements a message router that consumes messages from an MQTT broker, classifies them based on their content and metadata, and routes them to corresponding Kafka topics and MQTT topics as CloudEvents.

#### Structs
- **MessageRouter:** Represents the message router, which manages the Kafka producer and MQTT client.

#### Implementations
- **MessageRouter:**
  - **new:** Creates a new instance of MessageRouter with the given Kafka brokers and MQTT broker.
  - **start:** Starts the message routing process by subscribing to an MQTT topic and continuously handling incoming messages.
  - **extract_metadata:** Extracts metadata from the received message.
  - **classify_and_route_message:** Classifies the message based on its content and metadata, and routes it to the appropriate Kafka topic and MQTT topic as a CloudEvent.
  - **parse_message:** Parses the message using spaCy and generates an entity graph.
  - **classify_message:** Classifies the message based on the entity graph and metadata.
  - **route_message:** Routes the message to the corresponding Kafka topic and MQTT topic based on its classification.
  - **create_cloudevent:** Creates a CloudEvent with the given classification and message content.

#### Methods
- **MessageRouter::new:** Creates a new instance of MessageRouter with the given Kafka brokers and MQTT broker.
- **MessageRouter::start:** Starts the message routing process by subscribing to an MQTT topic and continuously handling incoming messages.
- **MessageRouter::extract_metadata:** Extracts metadata from the received message.
- **MessageRouter::classify_and_route_message:** Classifies the message based on its content and metadata, and routes it to the appropriate Kafka topic and MQTT topic as a CloudEvent.
- **MessageRouter::parse_message:** Parses the message using spaCy and generates an entity graph.
- **MessageRouter::classify_message:** Classifies the message based on the entity graph and metadata.
- **MessageRouter::route_message:** Routes the message to the corresponding Kafka topic and MQTT topic based on its classification.
- **MessageRouter::create_cloudevent:** Creates a CloudEvent with the given classification and message content.

#### Dependencies
- **cloudevents:** Used for creating and serializing CloudEvents.
- **entity_graph:** Used for representing and manipulating entity graphs.
- **kafka_producer:** Used for producing messages to Kafka topics.
- **message_metadata:** Used for representing message metadata.
- **rumqttc:** Used for connecting to an MQTT broker and subscribing to topics.
- **serde_json:** Used for JSON serialization and deserialization.
- **spacy:** Used for natural language processing and entity extraction.
- **std::collections::HashMap:** Used for storing and accessing metadata key-value pairs.

#### Tests
- **test_classify_and_route_message:** Tests the classify_and_route_message method by creating a MessageRouter instance, classifying a message with specific metadata, and verifying that the message is routed correctly.

# Language Detection

## Path: src/nlu/detect_language.rs
- **Description:** Provides functionality for detecting the language of a given utterance using the Spacy and Rasa libraries.

### Functions
- **detect_languages(utterance: &str) -> PyResult<Vec>**
  - **Description:** Detects the languages of the given utterance using Spacy and Rasa libraries.
  - **Arguments:**
    - `utterance:` A string slice containing the text for language detection.
  - **Returns:** A PyResult wrapping a vector of detected languages as strings.
  - **Functionality:**
    - Acquires the Global Interpreter Lock (GIL) and gets the Python interpreter instance.
    - Imports and initializes Spacy for language processing.
    - Imports and initializes Rasa for language detection.
    - Processes the utterance using Spacy to create a document object.
    - Temporarily disables named entity recognition for efficiency.
    - Detects language(s) using the Rasa language detector on the processed document.
    - Extracts and returns the detected languages.
    - (Novel implementation) Detects language confidence scores using the Rasa language detector.
    - Prints the detected languages and their confidence scores.

### Dependencies
- **rust_cpython:** Used to interface with Python for language detection using Spacy and Rasa.
- **pyo3:** Used for Python integration and interoperability with Rust.

# TED Policy Configuration

## Path
src/nlu/ted_policy_configuration.rs

## Description
Defines the configuration struct and associated methods for the TED (Transformer Embedding Dialogue) policy in the Natural Language Understanding (NLU) module.

### Structs
- **TedPolicyConfig:** Represents the configuration for the TED policy, with optional fields for the number of layers, hidden size, and learning rate.

### Implementations
- **TedPolicyConfig:**
  - **new:** Creates a new instance of TedPolicyConfig with all fields set to None.
  - **set_num_layers:** Sets the number of layers in the TED policy configuration.
  - **set_hidden_size:** Sets the hidden size in the TED policy configuration.
  - **set_learning_rate:** Sets the learning rate in the TED policy configuration.
  - **build:** Generates a string representation of the current TED policy configuration and sends it as a message using the provided Sender.

- **FromAction for TedPolicyConfig:** Implements the FromAction trait for TedPolicyConfig, allowing it to be created from specific action strings.
  - **from_action:** Creates a new instance of TedPolicyConfig based on the provided action string. Supported actions are "set_num_layers", "set_hidden_size", "set_learning_rate", and "build".

- **FromSender for TedPolicyConfig:** Implements the FromSender trait for TedPolicyConfig, allowing it to be created from a Sender.
  - **from_sender:** Creates a new instance of TedPolicyConfig from the provided Sender.

### Methods
- **TedPolicyConfig::new:** Creates a new instance of TedPolicyConfig with all fields set to None.
- **TedPolicyConfig::set_num_layers:** Sets the number of layers in the TED policy configuration.
- **TedPolicyConfig::set_hidden_size:** Sets the hidden size in the TED policy configuration.
- **TedPolicyConfig::set_learning_rate:** Sets the learning rate in the TED policy configuration.
- **TedPolicyConfig::build:** Generates a string representation of the current TED policy configuration and sends it as a message using the provided Sender.
- **FromAction::from_action:** Creates a new instance of TedPolicyConfig based on the provided action string.
- **FromSender::from_sender:** Creates a new instance of TedPolicyConfig from the provided Sender.

### Dependencies
- **rasa_rust_sdk:** Provides the FromAction and FromSender traits, as well as the Sender type for sending messages.

### Additional Notes
- The TedPolicyConfig struct is used to configure the TED policy in the NLU module.
- The configuration includes optional fields for the number of layers, hidden size, and learning rate.
- The build method generates a string representation of the current configuration and sends it as a message using the provided Sender.
- The FromAction and FromSender traits allow TedPolicyConfig to be created from specific action strings or a Sender, respectively.

# Text Processing Pipeline Documentation

## Modules

### Text Processing Pipeline
- **Path:** src/nlu/text_pipeline.rs
- **Description:** Defines a text processing pipeline for natural language understanding tasks, such as language detection, tokenization, lemmatization, stopword removal, and stemming.

#### Structs
- **TextProcessingPipeline:** Represents the text processing pipeline, which holds a vector of processing steps.

#### Implementations
- **TextProcessingPipeline:**
  - **new:** Creates a new instance of TextProcessingPipeline with the given processing steps.
  - **process:** Applies the processing steps to the input text and returns the processed tokens.

#### Functions
- **detect_language:** Detects the language of the input text using the provided Language object and returns the detected language as a string.
- **tokenize_text:** Tokenizes the input text using the provided Language object and returns a vector of tokens.
- **lemmatize_text:** Lemmatizes the input tokens using the provided Language object and returns a vector of lemmatized tokens.
- **remove_stopwords:** Removes stopwords from the input tokens using the provided Language object and returns a vector of filtered tokens.
- **stem_text:** Stems the input tokens using the provided Language object and returns a vector of stemmed tokens.

#### Dependencies
- **petgraph:** Used for creating and manipulating graphs to represent the text processing pipeline.
- **text_processor:** A custom library that provides the Language struct and associated methods for text processing tasks.

# Text Processors Documentation

## Modules

### Text Processors
- **Path:** src/nlu/text_processors.rs
- **Description:** Provides a collection of text processing utilities for natural language understanding (NLU).

#### Structs
- **TextProcessor:** Represents a text processor that applies a pipeline of processing steps to input text.

#### Implementations
- **TextProcessor:**
  - **new:** Creates a new instance of TextProcessor with the given pipeline of processing steps.
  - **process_text:** Processes the input text using the defined pipeline and returns a HashMap containing the original text, processed text, classification, named entities, lemmas, and the result of each processing step.
  - **process_text_all_permutations:** Processes the input text using all possible permutations of the pipeline steps and returns a vector of HashMaps, each containing the results for a specific permutation.
  - **process_pipeline:** Applies the pipeline steps sequentially to the input text and returns the processed text.
  - **process_pipeline_permutation:** Applies a specific permutation of the pipeline steps to the input text and returns the processed text.

#### Functions
- **get_text_processing_permutations:** Generates all possible permutations of the given pipeline steps.
- **get_function_name:** Retrieves the name of a given function as a string (placeholder implementation).
- **process_message:** Processes a message using a delegate, knowledge agent, and Q-learning agent, and generates a response based on the extracted information and selected action.

#### Dependencies
- **crate::bindings::spacy_bindings:** Provides bindings to the Spacy library for natural language processing tasks.
- **std::collections::HashMap:** Used for storing and retrieving key-value pairs.

# AI Provider Module

## Path: src/provider_types/ai.rs
- **Description:** Provides functionality for interacting with AI providers and managing AI-related tasks.

### Structs
- **InferenceRequest:** Represents a request for running inference on a message using an AI provider.
    - `message`: The message to run inference on.
    - `model`: Optional model to use for inference.
    - `parameters`: Optional parameters for the inference request.
- **InferenceResponse:** Represents the response from running inference on a message.
    - `message`: The processed message.
    - `confidence`: Optional confidence score of the inference.
    - `model_used`: Optional model used for the inference.
- **GenerationRequest:** Represents a request for generating a message using an AI provider.
    - `message`: The input message for generation.
    - `max_length`: Optional maximum length of the generated message.
    - `temperature`: Optional temperature for controlling the randomness of the generated message.
    - `n_best`: Optional number of best generated messages to return.
- **GenerationResponse:** Represents the response from generating a message.
    - `message`: The generated message.
    - `model_used`: Optional model used for the generation.
- **AIProvider:** Implements the AIProviderTrait for interacting with an AI provider's API.
    - `api_key`: The API key for authentication.
    - `client`: The HTTP client for making requests to the AI provider's API.
    - `base_url`: The base URL of the AI provider's API.
- **ProviderInfo:** Represents information about an AI provider.
    - `name`: The name of the AI provider.
    - `description`: A description of the AI provider.
    - `capabilities`: A list of capabilities supported by the AI provider.
- **ProviderSelector:** Manages multiple AI providers and selects the appropriate provider based on criteria.
    - `providers`: A map of AI providers keyed by their identifier.
- **AIProviderManager:** Orchestrates the usage of AI providers for running inference and generation tasks.
    - `provider_selector`: The ProviderSelector instance for selecting AI providers.
    - `default_provider_key`: The default AI provider key to use when no suitable provider is found.
    - `message_router`: The MessageRouter instance for routing generated messages.

### Traits
- **AIProviderTrait:** Defines the interface for an AI provider.
    - `run_inference`: Runs inference on a message using the AI provider.
    - `run_generation`: Generates a message using the AI provider.
    - `get_provider_info`: Retrieves information about the AI provider.

### Implementations
- **AIProviderTrait for AIProvider:** Implements the AIProviderTrait for AIProvider.
    - `run_inference`: Sends an inference request to the AI provider's API and returns the response.
    - `run_generation`: Sends a generation request to the AI provider's API and returns the response.
    - `get_provider_info`: Retrieves information about the AI provider from the API.

### Methods
- **AIProviderManager::new:** Creates a new instance of AIProviderManager with the provided API keys and default provider key.
- **AIProviderManager::run_inference:** Runs inference on a message using the selected AI provider based on the message classification.
- **AIProviderManager::run_generation:** Generates a message using the selected AI provider based on the message classification.
- **ProviderSelector::new:** Creates a new instance of ProviderSelector.
- **ProviderSelector::add_provider:** Adds an AI provider to the ProviderSelector with the given key.
- **ProviderSelector::select_provider:** Selects the most suitable AI provider based on the provided criteria and message.

### Dependencies
- **crate::messaging:** Provides the Message, classify_message, and MessageRouter types.
- **reqwest:** Used for making HTTP requests to the AI provider's API.
- **serde:** Used for serializing and deserializing request and response structs.
- **std::collections::HashMap:** Used for storing AI providers and criteria.
- **std::sync::Arc:** Used for thread-safe reference counting of AI providers.


# Charts Provider Documentation

## Modules

### Charts Provider
- **Path:** src/provider_types/charts.rs
- **Description:** Defines the `ChartsProvider` struct and its associated methods for creating and managing charts using the plotters library.

#### Structs
- **ChartsProvider:** Represents a provider for creating and managing charts.

#### Implementations
- **ChartsProvider:**
  - **new:** Creates a new instance of `ChartsProvider`.
  - **create_line_chart:** Creates a line chart with the given data and configuration.
    - **data:** A vector of tuples representing the data points for the line chart. Each tuple contains an `f64` value for the x-coordinate and a corresponding `f64` value for the y-coordinate.
    - **config:** An optional `LineChartConfig` struct containing the configuration options for the line chart, such as the chart title, axis labels, and legend.
    - **Returns:** A `Result<(), Box<dyn std::error::Error>>` indicating the success or failure of the chart creation process.
  - **create_bar_chart:** Creates a bar chart with the given data and configuration.
    - **data:** A vector of tuples representing the data points for the bar chart. Each tuple contains a `String` value for the category label and a corresponding `f64` value for the bar height.
    - **config:** An optional `BarChartConfig` struct containing the configuration options for the bar chart, such as the chart title, axis labels, and bar colors.
    - **Returns:** A `Result<(), Box<dyn std::error::Error>>` indicating the success or failure of the chart creation process.
  - **create_pie_chart:** Creates a pie chart with the given data and configuration.
    - **data:** A vector of tuples representing the data points for the pie chart. Each tuple contains a `String` value for the category label and a corresponding `f64` value for the slice percentage.
    - **config:** An optional `PieChartConfig` struct containing the configuration options for the pie chart, such as the chart title and legend.
    - **Returns:** A `Result<(), Box<dyn std::error::Error>>` indicating the success or failure of the chart creation process.
  - **save_chart:** Saves the chart to a file with the specified file path.
    - **file_path:** A `String` representing the file path where the chart should be saved.
    - **Returns:** A `Result<(), Box<dyn std::error::Error>>` indicating the success or failure of the chart saving process.

#### Methods
- **ChartsProvider::new:** Creates a new instance of `ChartsProvider`.
- **ChartsProvider::create_line_chart:** Creates a line chart with the given data and configuration.
- **ChartsProvider::create_bar_chart:** Creates a bar chart with the given data and configuration.
- **ChartsProvider::create_pie_chart:** Creates a pie chart with the given data and configuration.
- **ChartsProvider::save_chart:** Saves the chart to a file with the specified file path.

#### Dependencies
- **plotters:** A powerful Rust library for creating various types of charts and plots.
- **std::error:** The standard library's error handling module.

### Configuration Structs
- **LineChartConfig:** Represents the configuration options for a line chart.
- **BarChartConfig:** Represents the configuration options for a bar chart.
- **PieChartConfig:** Represents the configuration options for a pie chart.

# Messaging Provider Types Documentation

## Modules

### Messaging Provider Types
- **Path:** src/provider_types/messaging.rs
- **Description:** Defines the messaging provider types, traits, and implementations for sending messages and subscribing to events using different messaging providers (REST, gRPC, and Webhooks).

#### Enums
- **MessagingProviderType:** Represents the different types of messaging providers (REST, gRPC, and Webhooks).

#### Structs
- **Message:** Represents a message with content, metadata, and an entity graph.
- **MessagingProviderFactory:** A factory for creating messaging providers based on the provider type and configuration.
- **RestMessagingProvider:** Implements the MessagingProvider trait for REST-based messaging.
- **GrpcMessagingProvider:** Implements the MessagingProvider trait for gRPC-based messaging.
- **WebhooksMessagingProvider:** Implements the MessagingProvider trait for Webhooks-based messaging.

#### Traits
- **MessagingProvider:** Defines the interface for messaging providers, including sending messages and subscribing to events.

#### Implementations
- **MessagingProviderFactory:**
  - **create_messaging_provider:** Creates a messaging provider based on the provider type and configuration.

- **MessagingProvider for RestMessagingProvider:**
  - **send_message:** Sends a message to a recipient using the REST messaging provider.
  - **subscribe_events:** Subscribes to events using the REST messaging provider.

- **MessagingProvider for GrpcMessagingProvider:**
  - **send_message:** Sends a message to a recipient using the gRPC messaging provider.
  - **subscribe_events:** Subscribes to events using the gRPC messaging provider.

- **MessagingProvider for WebhooksMessagingProvider:**
  - **send_message:** Sends a message to a recipient using the Webhooks messaging provider.
  - **subscribe_events:** Subscribes to events using the Webhooks messaging provider.

#### Methods
- **MessagingProviderFactory::create_messaging_provider:** Creates a messaging provider based on the provider type and configuration.
- **RestMessagingProvider::send_message:** Sends a message to a recipient using the REST messaging provider.
- **RestMessagingProvider::subscribe_events:** Subscribes to events using the REST messaging provider.
- **GrpcMessagingProvider::send_message:** Sends a message to a recipient using the gRPC messaging provider.
- **GrpcMessagingProvider::subscribe_events:** Subscribes to events using the gRPC messaging provider.
- **WebhooksMessagingProvider::send_message:** Sends a message to a recipient using the Webhooks messaging provider.
- **WebhooksMessagingProvider::subscribe_events:** Subscribes to events using the Webhooks messaging provider.

#### Dependencies
- **async_trait:** Used for defining async traits.
- **std::collections::HashMap:** Used for storing configuration key-value pairs.
- **super::data_exchange::{ConnectionInfo, DataExchange}:** Imports the ConnectionInfo and DataExchange types from the data_exchange module.


# Payment Providers

## Path: src/provider_types/payments.rs
## Description: Defines traits and structs for different payment provider implementations.

### Traits
- **PaymentProvider:** Defines the interface for payment provider implementations.
  - **pay:** Processes a payment with the given amount and currency.
  - **refund:** Processes a refund for a given payment ID.
  - **get_payment_status:** Retrieves the status of a payment with the given ID.
  - **get_payment_history:** Retrieves the payment history for a given user ID.

### Structs
- **Payment:** Represents a single payment transaction.
  - **id:** The unique identifier of the payment.
  - **user_id:** The ID of the user associated with the payment.
  - **amount:** The amount of the payment.
  - **currency:** The currency of the payment.
  - **status:** The current status of the payment.
  - **created_at:** The timestamp when the payment was created.
- **StripePaymentProvider:** Implements the PaymentProvider trait for Stripe payment processing.
  - **api_key:** Holds the Stripe API key.
  - **new:** Creates a new instance of StripePaymentProvider with the given API key.
  - **pay:** Processes a payment using the Stripe API with the given amount and currency.
  - **refund:** Processes a refund using the Stripe API for a given payment ID.
  - **get_payment_status:** Retrieves the status of a payment from the Stripe API with the given ID.
  - **get_payment_history:** Retrieves the payment history from the Stripe API for a given user ID.

- **PayPalPaymentProvider:** Implements the PaymentProvider trait for PayPal payment processing.
  - **client_id:** Holds the PayPal client ID.
  - **client_secret:** Holds the PayPal client secret.
  - **new:** Creates a new instance of PayPalPaymentProvider with the given client ID and secret.
  - **pay:** Processes a payment using the PayPal API with the given amount and currency.
  - **refund:** Processes a refund using the PayPal API for a given payment ID.
  - **get_payment_status:** Retrieves the status of a payment from the PayPal API with the given ID.
  - **get_payment_history:** Retrieves the payment history from the PayPal API for a given user ID.

### Enums
- **Currency:** Represents different currency types (USD, EUR, GBP, etc.).
- **PaymentStatus:** Represents different payment statuses (Pending, Completed, Failed, Refunded).

### Type Aliases
- **PaymentResult:** A Result type for payment operations, with PaymentError as the error variant.
- **PaymentHistory:** A vector of Payment structs representing a user's payment history.

### Search Provider
- **Path:** src/provider_types/search.rs
- **Description:** Defines the search provider trait and implementations for different search providers.

#### Traits
- **SearchProvider:** Defines the interface for search providers.
    - **search:** Performs a search query and returns the results as a HashMap of key-value pairs.

#### Structs
- **SearchProviderFactory:** A factory struct for creating search provider instances based on the provider type and configuration.
- **WikipediaSearchProvider:** An implementation of the SearchProvider trait for searching Wikipedia.
    - **api_url:** The URL of the Wikipedia API.
    - **knowledge_graph:** A HashMap representing the knowledge graph, mapping categories to topics and their associated weights.

#### Implementations
- **SearchProviderFactory:**
    - **create_provider:** Creates a new instance of a search provider based on the provider type and configuration. It matches the provider type and calls the corresponding constructor with the provided configuration.
- **WikipediaSearchProvider:**
    - **new:** Creates a new instance of WikipediaSearchProvider with the given API URL. It initializes the knowledge graph with predefined categories, topics, and weights.
    - **update_knowledge_graph:** Updates the knowledge graph by inserting or updating the weight of a topic within a category.
    - **get_topic_weight:** Retrieves the weight of a topic within a category from the knowledge graph.
    - **calculate_query_relevance:** Calculates the relevance of a query based on the topics and weights in the knowledge graph.
- **SearchProvider for WikipediaSearchProvider:**
    - **search:** Performs a search query on Wikipedia and returns the results. This is a placeholder implementation that returns an empty HashMap.

#### Methods
- **SearchProviderFactory::create_provider:** Creates a new instance of a search provider based on the provider type and configuration.
- **WikipediaSearchProvider::new:** Creates a new instance of WikipediaSearchProvider with the given API URL and initializes the knowledge graph.
- **WikipediaSearchProvider::update_knowledge_graph:** Updates the knowledge graph by inserting or updating the weight of a topic within a category.
- **WikipediaSearchProvider::get_topic_weight:** Retrieves the weight of a topic within a category from the knowledge graph.
- **WikipediaSearchProvider::calculate_query_relevance:** Calculates the relevance of a query based on the topics and weights in the knowledge graph.
- **SearchProvider::search:** Performs a search query and returns the results as a HashMap of key-value pairs.

#### Dependencies
- **std::collections::HashMap:** Used for storing the knowledge graph and search results.
- **crate::data_exchange::DataExchange:** Used for data exchange functionality (not directly used in this file but mentioned in the code).

# Event Recommendations Documentation

## Modules

### Event Recommendations
- **Path:** src/recommendations/event_recommendations.rs
- **Description:** Implements a recommendation system for suggesting events to users based on their preferences, location, and the timing of the events.

#### Structs
- **RecommendHandler:** Responsible for recommending events to users based on their preferences and proximity.
- **Event:** Represents an event with its details such as ID, name, location, start time, end time, significance, event type, and attributes.
- **Alert:** Represents an alert containing the event name, message, and the associated event.
- **EventCandidate:** Represents a candidate event with its details, distance from the user, user preference, and a potential filter reason.

#### Enums
- **RecommendError:** Custom error type for recommendation-related errors, including data conversion errors and Neo4j errors.
- **EventType:** Represents different types of events (e.g., ScheduledEvent, CasualMeetup, Conference, Workshop).
- **CandidateFilterReason:** Represents reasons for filtering out event candidates (e.g., Unschedulable, TooFar, TooLate).

#### Implementations
- **RecommendHandler:**
  - **new:** Creates a new instance of RecommendHandler with the given Neo4j client, distance threshold, and time-to-start threshold.
  - **recommend_event:** Recommends events to a user based on their location, time, and preferences. It follows a series of stages: recall, dependency loading, filtering, and sorting.
  - **generate_message:** Generates a personalized message for the user based on the recommended event details (to be implemented with a generative language model).
  - **recommend_recall:** Retrieves a list of potential event candidates based on user preferences and proximity using a Neo4j database query.
  - **load_event_dependencies:** Loads additional dependencies for each event candidate, such as checking event schedulability and other logistical considerations.
  - **filter_event_candidates:** Filters out unsuitable event candidates based on the filter reasons determined during the recall and dependency loading stages.
  - **sort_events:** Sorts the remaining event candidates based on a combination of user preferences and event significance.

#### Methods
- **RecommendHandler::recommend_event:** Recommends events to a user based on their location, time, and preferences. It follows a series of stages: recall, dependency loading, filtering, and sorting.
- **RecommendHandler::generate_message:** Generates a personalized message for the user based on the recommended event details (to be implemented with a generative language model).
- **RecommendHandler::recommend_recall:** Retrieves a list of potential event candidates based on user preferences and proximity using a Neo4j database query.
- **RecommendHandler::load_event_dependencies:** Loads additional dependencies for each event candidate, such as checking event schedulability and other logistical considerations.
- **RecommendHandler::filter_event_candidates:** Filters out unsuitable event candidates based on the filter reasons determined during the recall and dependency loading stages.
- **RecommendHandler::sort_events:** Sorts the remaining event candidates based on a combination of user preferences and event significance.

#### Dependencies
- **crate::bindings::spacy_bindings:** Provides data structures for handling entity and label information from user preferences.
- **crate::graphs::event_graph:** Defines the event and location data structures and their interactions.
- **futures:** Used for handling asynchronous operations.
- **neo4rs:** A Rust client for Neo4j database operations.
- **std::sync::Arc:** Enables shared ownership of the Neo4j client across multiple tasks.
- **std::convert::TryFrom:** Used for converting between data types.
- **std::collections::HashMap:** Used for storing event attributes and message data.
- **thiserror::Error:** Used for defining custom error types.

# Rules Engine Documentation

## Modules

### Rules Engine
- **Path:** src/rules/rules.rs
- **Description:** Implements a rules engine for evaluating and executing rules based on incoming events and facts.

#### Structs
- **Rule:** Represents a rule with a name, description, condition, and action.
- **RuleEngine:** Represents the rules engine, which manages a collection of rules and executes them based on incoming events and facts.

#### Enums
- **RuleError:** Defines the possible errors that can occur during rule evaluation and execution.

#### Implementations
- **Rule:**
  - **new:** Creates a new instance of Rule with the given name, description, condition, and action.
  - **evaluate:** Evaluates the rule's condition against the provided facts and returns a boolean result.
  - **execute:** Executes the rule's action if the condition is satisfied.
- **RuleEngine:**
  - **new:** Creates a new instance of RuleEngine with an empty set of rules.
  - **add_rule:** Adds a new rule to the rules engine.
  - **remove_rule:** Removes a rule from the rules engine by its name.
  - **get_rule:** Retrieves a rule from the rules engine by its name.
  - **execute_rules:** Executes all the rules in the rules engine against the provided facts and returns a vector of executed rule names and their results.
  - **execute_rule:** Executes a specific rule by its name against the provided facts and returns the result.
  - **clear_rules:** Clears all the rules from the rules engine.

#### Methods
- **Rule::new:** Creates a new instance of Rule with the given name, description, condition, and action.
- **Rule::evaluate:** Evaluates the rule's condition against the provided facts and returns a boolean result.
- **Rule::execute:** Executes the rule's action if the condition is satisfied.
- **RuleEngine::new:** Creates a new instance of RuleEngine with an empty set of rules.
- **RuleEngine::add_rule:** Adds a new rule to the rules engine.
- **RuleEngine::remove_rule:** Removes a rule from the rules engine by its name.
- **RuleEngine::get_rule:** Retrieves a rule from the rules engine by its name.
- **RuleEngine::execute_rules:** Executes all the rules in the rules engine against the provided facts and returns a vector of executed rule names and their results.
- **RuleEngine::execute_rule:** Executes a specific rule by its name against the provided facts and returns the result.
- **RuleEngine::clear_rules:** Clears all the rules from the rules engine.

#### Dependencies
- **std::collections::HashMap:** Used for storing the rules in the RuleEngine.
- **std::error::Error:** Used for defining the RuleError enum.
- **std::fmt:** Used for formatting error messages.
- **serde::{Deserialize, Serialize}:** Used for serializing and deserializing Rule structs.

# Event Significance Documentation

## Modules

### Event Significance
- **Path:** src/significance/event_significance.rs
- **Description:** Defines structures and methods for representing and calculating the significance of events based on their type and associated attributes.

#### Enums
- **EventType:** Represents the different types of events that can occur.
  - **SeismicAnomaly:** A seismic event.
  - **ScheduledEvent:** A scheduled event.
  - **AgentPreference:** A preference expressed by an agent.
  - **AlertGraph:** An alert on a graph.
  - **CustomEvent(String):** A custom event type.

#### Structs
- **EventSignificance:** Represents the significance of an event, which is determined by its attributes and type.
  - **attributes:** A HashMap containing the attributes associated with the event and their values.
  - **event_type:** The type of the event.
- **EventCollection:** Represents a collection of events and their significances.
  - **events:** A vector of EventSignificance instances.

#### Implementations
- **EventSignificance:**
  - **new:** Constructs a new instance of EventSignificance with the given event type and attributes.
  - **event_type:** Returns a reference to the event type.
  - **attribute:** Returns the value of a particular attribute associated with the event, if it exists.
  - **calculate_significance:** Calculates the significance of the event based on its type and attributes.
    - For a SeismicAnomaly event, the significance is based on the magnitude, depth, and location factor of the event.
    - For a ScheduledEvent, the significance is based on its importance and urgency.
    - For an AgentPreference event, the significance is based on the preference expressed and the agent's influence.
    - For an AlertGraph event, the significance is based on the severity of the alert, its duration, and the graph's importance.
    - For a CustomEvent, the significance is based on a custom calculation using a custom factor.
  - **update_attribute:** Updates the value of an attribute.
  - **remove_attribute:** Removes an attribute from the event.
  - **has_attribute:** Checks if an attribute exists for the event.

- **EventCollection:**
  - **new:** Constructs a new instance of EventCollection with the given vector of events.
  - **add_event:** Adds an event to the collection.
  - **remove_event:** Removes an event from the collection by index.
  - **total_significance:** Calculates the total significance of all events in the collection.
  - **events_by_type:** Returns a vector of references to the events of a particular type.
  - **most_significant_event:** Returns a reference to the event with the highest significance in the collection.

#### Methods
- **EventSignificance::new:** Constructs a new instance of EventSignificance with the given event type and attributes.
- **EventSignificance::event_type:** Returns a reference to the event type.
- **EventSignificance::attribute:** Returns the value of a particular attribute associated with the event, if it exists.
- **EventSignificance::calculate_significance:** Calculates the significance of the event based on its type and attributes.
- **EventSignificance::update_attribute:** Updates the value of an attribute.
- **EventSignificance::remove_attribute:** Removes an attribute from the event.
- **EventSignificance::has_attribute:** Checks if an attribute exists for the event.
- **EventCollection::new:** Constructs a new instance of EventCollection with the given vector of events.
- **EventCollection::add_event:** Adds an event to the collection.
- **EventCollection::remove_event:** Removes an event from the collection by index.
- **EventCollection::total_significance:** Calculates the total significance of all events in the collection.
- **EventCollection::events_by_type:** Returns a vector of references to the events of a particular type.
- **EventCollection::most_significant_event:** Returns a reference to the event with the highest significance in the collection.
