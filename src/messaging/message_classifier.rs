use crate::messaging::message_metadata::MetadataValue;
use crate::graphs::nl_to_graph::{EntityGraph, EntityType};

use std::collections::HashMap;

pub fn classify_message(metadata: &HashMap<String, MetadataValue>, entity_graph: &dyn EntityGraph) -> String {
    let mut classification = String::new();

    if entity_graph.has_entities_of_type(&EntityType::Location) {
        classification = "Location-based message".to_string();
    } else if let Some(MetadataValue::ReplyInfo(_)) = metadata.get("reply_to") {
        classification = "Reply message".to_string();
    } else if let Some(MetadataValue::MediaAttachment(_)) = metadata.get("media") {
        classification = "Media message".to_string();
    } else if let Some(MetadataValue::Bool(true)) = metadata.get("post") {
        classification = "Post message".to_string();
    } else if let Some(MetadataValue::Bool(true)) = metadata.get("pinned") {
        classification = "Pinned message".to_string();
    } else {
        classification = "Regular message".to_string();
    }

    classification
}

/*
This module defines a function called classify_message. The function takes two arguments: a metadata HashMap that contains metadata information about a message, and an entity_graph that represents the entities in the message.

The function first initializes an empty classification string. It then checks if there are any entities of type Location in the entity_graph. If so, it sets the classification to "Location-based message".

If there is no Location entity, the function checks if the message is a reply. If the metadata HashMap has a reply_to key with a MetadataValue of type ReplyInfo, the function sets the classification to "Reply message".

If the message is not a reply, the function checks if the message contains any media attachments. If the metadata HashMap has a media key with a MetadataValue of type MediaAttachment, the function sets the classification to "Media message".

If the message is not a media message, the function checks if the message is a post. If the metadata HashMap has a post key with a MetadataValue of true, the function sets the classification to "Post message".

If the message is not a post, the function checks if the message is pinned. If the metadata HashMap has a pinned key with a MetadataValue of true, the function sets the classification to "Pinned message".

Finally, if the message does not fit any of the above classifications, the function sets the classification to "Regular message". The function returns the classification string as its output.
 */