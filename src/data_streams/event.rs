use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use futures::future::join_all;
use lazy_static::lazy_static;
use tokio::sync::Mutex;


lazy_static! {
    pub static ref EVENT_MANAGER: Arc<Mutex<EventHandlers>> =
        Arc::new(Mutex::new(EventHandlers::default()));
}

pub async fn register_event_handler(handler: Box<dyn EventHandler>) {
    EVENT_MANAGER.lock().await.register(handler).await;
}

pub async fn handle_event(event: Event) {
    EVENT_MANAGER.lock().await.handle(event);
}

#[derive(Debug, Copy, Clone)]
pub struct Location(pub f32, pub f32, pub f32);

#[derive(Debug, Copy, Clone)]
pub struct Duration(pub u64, pub u64);


#[derive(Debug, Clone)]
pub struct Event {
    #[allow(unused)]
    pub unique_id: String,
    #[allow(unused)]
    pub user_id: Option<i64>,
    #[allow(unused)]
    pub time: i64,
    #[allow(unused)]
    pub header: EventHeader,
    #[allow(unused)]
    pub event_type: EventType,

    pub id: i64,
    pub name: String,
    pub location: Location,
    pub start_time: u64,
    pub end_time: u64,
    pub significance: f64,
    pub attributes: HashMap<String, f64>,
    pub duration: Duration,
    pub dependencies: Vec<Arc<Event>>,
    pub start: i32,
    pub end: i32,
    pub resource: String,
    pub tags: Vec<String>,
}


// the type of this event. i.e user made an utterance, user scheduled a plan
// or user posted a media in the dialogue, etc.
#[derive(Debug, Clone)]
pub enum EventType {
    Mentioned(String),
    Scheduled(String),
    PostedMedia(String),
    ScheduledEvent,
    CasualMeetup,
    Conference,
    Workshop,
}

// basic information that any types of event contain
#[derive(Debug, Clone, Default)]
pub struct EventHeader {
    #[allow(unused)]
    ip: Option<String>,
    #[allow(unused)]
    device_type: Option<String>,
    #[allow(unused)]
    trace_id: Option<String>,
    #[allow(unused)]
    via_bot_id: bool,
}

impl EventType {
    pub fn name(&self) -> &'static str {
        match self {
            EventType::Mentioned(_) => "Mentioned",
            EventType::Scheduled(_) => "Scheduled",
            EventType::PostedMedia(_) => "PostedMedia",
            EventType::ScheduledEvent => "ScheduledEvent",
            EventType::CasualMeetup => "CasualMeetup",
            EventType::Conference => "Conference",
            EventType::Workshop => "Workshop",
        }
    }
}

// manage the handlers that are registered with an event type.
// when an event is met, find the interested handlers and let them
// handle it.
#[derive(Default)]
pub struct EventHandlers {
    handlers: HashMap<&'static str, Arc<Mutex<Vec<Box<dyn EventHandler>>>>>,
}

impl EventHandlers {
    async fn register(&mut self, handler: Box<dyn EventHandler>) {
        self.handlers
            .entry(handler.event_name())
            .or_insert(Arc::new(Mutex::new(vec![])))
            .lock()
            .await
            .push(handler);
    }

    // for events with the same type, they should be handled in order.
    // while for events with difference types, they can be handled concurrently.
    pub fn handle(&mut self, event: Event) {
        if let Some(handlers) = self.handlers.get(event.event_type.name()) {
            let handlers = handlers.clone();
            tokio::spawn(async move {
                let mut handlers = handlers.lock().await;
                let mut tasks = vec![];
                for handler in handlers.iter_mut() {
                    tasks.push(handler.handle(event.clone()));
                }
                join_all(tasks).await.into_iter();
            });
        }
    }
}

#[async_trait]
pub trait EventHandler: Send {
    fn event_name(&self) -> &'static str;

    async fn handle(&mut self, event: Event);
}

mod test {
    use std::sync::Arc;
    use std::time::Duration;
    use std::collections::HashMap;
    use crate::event::Location;

    use async_trait::async_trait;
    use tokio::sync::Mutex;

    use crate::event::{
        handle_event, register_event_handler, Event, EventHandler, EventHeader, EventType,
    };

    #[tokio::test]
    async fn test_event_handler() {
        struct MentionedHandler {
            ids: Arc<Mutex<Vec<String>>>,
        }

        #[async_trait]
        impl EventHandler for MentionedHandler {
            fn event_name(&self) -> &'static str {
                "Mentioned"
            }

            async fn handle(&mut self, event: Event) {
                self.ids.lock().await.push(event.unique_id);
            }
        }

        struct ScheduledHandler {
            uids: Arc<Mutex<Vec<i64>>>,
        }

        #[async_trait]
        impl EventHandler for ScheduledHandler {
            fn event_name(&self) -> &'static str {
                "Scheduled"
            }

            async fn handle(&mut self, event: Event) {
                self.uids.lock().await.push(event.user_id.unwrap());
            }
        }
        let events: Vec<Event> = vec![
            EventType::Mentioned("Some utterances".to_string()),
            EventType::Scheduled("Go shopping tomorrow".to_string()),
        ]
        .into_iter()
        .map(|x| Event {
            unique_id: "UNIQUE_ID".to_string(),
            user_id: Some(1),
            time: 0,
            header: EventHeader::default(),
            event_type: x,
            id: 0,
            name: "".to_string(),
            location: Location(0.0, 0.0, 0.0),
            start_time: 0,
            end_time: 0,
            significance: 0.0,
            attributes: HashMap::new(),
            duration: Duration::from_secs(0),
            dependencies: Vec::new(),
            start: 0,
            end: 0,
            resource: "".to_string(),
            tags: Vec::new(),
        })
        .collect();
        
        let ids = Arc::new(Mutex::new(vec![]));
        let uids = Arc::new(Mutex::new(vec![]));
        let scheduled_handler =
            Box::new(ScheduledHandler { uids: uids.clone() }) as Box<dyn EventHandler>;
        let mentioned_handler =
            Box::new(MentionedHandler { ids: ids.clone() }) as Box<dyn EventHandler>;
        register_event_handler(scheduled_handler).await;
        register_event_handler(mentioned_handler).await;
        for event in events.into_iter() {
            handle_event(event).await;
        }
        // sleep to let the spawned tasks been executed
        tokio::time::sleep(Duration::from_secs(1)).await;
        let mut ids = ids.lock().await;
        assert_eq!(ids.pop(), Some("UNIQUE_ID".to_string()));
        assert!(ids.is_empty());
        let mut uids = uids.lock().await;
        assert_eq!(uids.pop(), Some(1));
        assert!(uids.is_empty());
    }
}
