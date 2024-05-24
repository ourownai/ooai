use std::collections::HashMap;
use crate::messaging::message_metadata;

#[derive(Debug)]
enum MetadataValue {
    Int(i32),
    String(String),
    Bool(bool),
    OptionInt(Option<i32>),
    ReplyInfo(Box<ReplyInfo>),
    MediaAttachment(Box<MediaAttachment>),
    Entities(Vec<MessageEntity>),
    Reactions(Vec<Reaction>),
}

#[derive(Debug)]
pub struct MessageMetadata {
    pub metadata: HashMap<String, MetadataValue>,
}

#[derive(Debug)]
struct ReplyInfo {
    // fields of the ReplyInfo struct
}

#[derive(Debug)]
struct MediaAttachment {
    // fields of the MediaAttachment struct
}

#[derive(Debug)]
struct MessageEntity {
    // fields of the MessageEntity struct
}

#[derive(Debug)]
struct Reaction {
    // fields of the Reaction struct
}

pub fn print_message_metadata() {
    let mut metadata = HashMap::new();

    // Insert metadata values into the HashMap
    metadata.insert("id".to_string(), MetadataValue::Int(123456));
    metadata.insert("flags".to_string(), MetadataValue::String("example flags".to_string()));
    metadata.insert("out".to_string(), MetadataValue::Bool(true));
    metadata.insert("mentioned".to_string(), MetadataValue::Bool(false));
    metadata.insert("media_unread".to_string(), MetadataValue::Bool(false));
    metadata.insert("silent".to_string(), MetadataValue::Bool(true));
    metadata.insert("post".to_string(), MetadataValue::Bool(false));
    metadata.insert("from_scheduled".to_string(), MetadataValue::Bool(false));
    metadata.insert("legacy".to_string(), MetadataValue::Bool(false));
    metadata.insert("edit_hide".to_string(), MetadataValue::Bool(false));
    metadata.insert("pinned".to_string(), MetadataValue::Bool(true));
    metadata.insert("noforwards".to_string(), MetadataValue::Bool(false));
    metadata.insert("from_id".to_string(), MetadataValue::Int(123));
    metadata.insert("peer_id".to_string(), MetadataValue::Int(456));
    metadata.insert("via_bot_id".to_string(), MetadataValue::OptionInt(Some(789)));
    metadata.insert("reply_to".to_string(), MetadataValue::ReplyInfo(Box::new(ReplyInfo { /* fields */ })));
    metadata.insert("date".to_string(), MetadataValue::Int(1645543200));
    metadata.insert("media".to_string(), MetadataValue::MediaAttachment(Box::new(MediaAttachment { /* fields */ })));
    metadata.insert("entities".to_string(), MetadataValue::Entities(vec![MessageEntity { /* fields */ }]));
    metadata.insert("views".to_string(), MetadataValue::Int(42));
    metadata.insert("forwards".to_string(), MetadataValue::Int(3));
    metadata.insert("replies".to_string(), MetadataValue::ReplyInfo(Box::new(ReplyInfo { /* fields */ })));
    metadata.insert("edit_date".to_string(), MetadataValue::OptionInt(Some(1645543210)));
    metadata.insert("post_author".to_string(), MetadataValue::String("example author".to_string()));
    metadata.insert("grouped_id".to_string(), MetadataValue::OptionInt(Some(987)));
    metadata.insert("reactions".to_string(), MetadataValue::Reactions(vec![Reaction { /* fields */ }]));
    metadata.insert("restriction_reason".to_string(), MetadataValue::String("example restriction".to_string()));
    metadata.insert("ttl_period".to_string(), MetadataValue::OptionInt(Some(60)));

    // Create a MessageMetadata struct from the metadata HashMap
    let message_metadata = MessageMetadata { metadata };

    // Print the message
    println!("{:#?}", message_metadata);
}

// The message schema is borrowed from Telegram and needs a refactor to fit our increased scope of use cases.

/*
This module creates a hashmap of message metadata values and then prints out the hashmap. It uses the types and logic defined in MessageMetaData.rs

It defines an enum called MetadataValue that represents the possible values that metadata fields can have. Each variant of the MetadataValue enum is associated with a different type of data, such as a boolean, string, integer, or a vector of other structs.

Next, four structs are defined: MessageMetadata, ReplyInfo, MediaAttachment, MessageEntity, and Reaction. The MessageMetadata struct contains a HashMap field of MetadataValue values. The other three structs represent different types of metadata values.

The print_message_metadata() function creates a new HashMap called metadata. We then insert a variety of metadata values into the hashmap, including integers, booleans, strings, vectors, and structs. Finally, the program creates a new MessageMetadata instance using the metadata hashmap and prints it out using the println!() macro.

The output is the entire message_metadata struct in a pretty-printed format that includes all of the metadata values and their associated types.
*/
