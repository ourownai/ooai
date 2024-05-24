use std::collections::HashMap;

#[derive(Debug)]
pub struct MessageMetadata {
    pub metadata: HashMap<String, MetadataValue>,
}

#[derive(Debug)]
pub enum MetadataValue {
    Bool(bool),
    String(String),
    Int(i64),
    OptionInt(Option<i64>),
    ReplyInfo(Box<ReplyInfo>),
    MediaAttachment(Box<MediaAttachment>),
    Entities(Vec<MessageEntity>),
    Reactions(Vec<Reaction>),
}

pub struct ReplyInfo {
    message_id: String,
    user_id: String,
    timestamp: i64,
}

pub struct MediaAttachment {
    media_type: String,
    url: String,
    thumbnail_url: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
    duration: Option<i32>,
}

pub struct MessageEntity {
    entity_type: String,
    offset: i32,
    length: i32,
    url: Option<String>,
    user: Option<User>,
}

pub struct User {
    user_id: String,
    username: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
}

pub struct Reaction {
    reaction: String,
    count: i32,
    users: Vec<String>,
}

impl MessageMetadata {
    pub fn new() -> Self {
        MessageMetadata {
            metadata: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: MetadataValue) {
        self.metadata.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&MetadataValue> {
        self.metadata.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<MetadataValue> {
        self.metadata.remove(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.metadata.len()
    }

    pub fn is_empty(&self) -> bool {
        self.metadata.is_empty()
    }

    pub fn clear(&mut self) {
        self.metadata.clear();
    }

    pub fn keys(&self) -> Vec<&String> {
        self.metadata.keys().collect()
    }

    pub fn values(&self) -> Vec<&MetadataValue> {
        self.metadata.values().collect()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<String, MetadataValue> {
        self.metadata.iter()
    }

    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<String, MetadataValue> {
        self.metadata.iter_mut()
    }
}