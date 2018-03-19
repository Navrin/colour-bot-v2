use std::sync::{Arc, Mutex};
use std::thread;

use serenity::model::channel::{Message, Reaction};
use serenity::model::id::{ChannelId, MessageId, UserId};

use futures::sync::mpsc::{channel, Receiver};

use parallel_event_emitter::{ListenerId, ParallelEventEmitter};

#[derive(Hash, Clone, Eq, PartialEq)]
pub enum CollectorItem {
    Message,
    Reaction,
}

#[derive(Clone)]
pub enum CollectorValue {
    Message(Message),
    Reaction(Reaction),
}

impl CollectorValue {
    fn author_id(&self) -> UserId {
        match self {
            &CollectorValue::Message(ref msg) => msg.author.id,
            &CollectorValue::Reaction(ref react) => react.user_id,
        }
    }

    fn channel_id(&self) -> ChannelId {
        match self {
            &CollectorValue::Message(ref msg) => msg.channel_id,
            &CollectorValue::Reaction(ref react) => react.channel_id,
        }
    }

    fn message_id(&self) -> MessageId {
        match self {
            &CollectorValue::Message(ref msg) => msg.id,
            &CollectorValue::Reaction(ref react) => react.message_id,
        }
    }
}

impl From<CollectorValue> for Message {
    fn from(value: CollectorValue) -> Self {
        match value {
            CollectorValue::Message(msg) => msg,
            CollectorValue::Reaction(_) => {
                panic!("Invariant! Expected a Message struct but got a reaction.")
            }
        }
    }
}

impl From<CollectorValue> for Reaction {
    fn from(value: CollectorValue) -> Self {
        match value {
            CollectorValue::Message(_) => {
                panic!("Invariant! Expect a Reaction struct but got a message.")
            }
            CollectorValue::Reaction(react) => react,
        }
    }
}

pub trait Collectible {
    fn collector_type() -> CollectorItem;
}

impl Collectible for Message {
    fn collector_type() -> CollectorItem {
        CollectorItem::Message
    }
}

impl Collectible for Reaction {
    fn collector_type() -> CollectorItem {
        CollectorItem::Reaction
    }
}

pub struct Collector(pub Arc<Mutex<ParallelEventEmitter<CollectorItem>>>);

impl Collector {
    pub fn new() -> Self {
        Collector(Arc::new(Mutex::new(ParallelEventEmitter::new())))
    }

    #[allow(dead_code)]
    pub fn get_custom(&self) -> CustomCollector {
        CustomCollector::new(self.0.clone())
    }
}

struct InnerCustomCollector {
    collector: Arc<Mutex<ParallelEventEmitter<CollectorItem>>>,
    listener_id: Option<ListenerId>,
    target_channel: Option<ChannelId>,
    target_user: Option<UserId>,
    target_message: Option<MessageId>,
    limit: usize,
    // count towards the limit so we know when to disconnect the listener.
    count: usize,
}

pub struct CustomCollector {
    inner: Arc<Mutex<InnerCustomCollector>>,
}

macro_rules! get_inner {
    ($inn: expr) => {{
        $inn.lock().expect("Error locking inner in get_inner!")
    }};
}

#[allow(dead_code)]
impl CustomCollector {
    pub fn new(collector: Arc<Mutex<ParallelEventEmitter<CollectorItem>>>) -> Self {
        CustomCollector {
            inner: Arc::new(Mutex::new(InnerCustomCollector {
                collector,
                listener_id: None,
                target_channel: None,
                target_user: None,
                target_message: None,
                limit: 1,
                count: 0,
            })),
        }
    }

    /// Collector will only get messages from this channel.
    pub fn set_channel(&self, chan: ChannelId) -> &Self {
        let mut inner = get_inner!(self.inner);
        inner.target_channel = Some(chan);

        self
    }

    /// Collector will only get messages form this user.
    pub fn set_author(&self, user: UserId) -> &Self {
        let mut inner = get_inner!(self.inner);
        inner.target_user = Some(user);

        self
    }

    pub fn set_message(&self, msg: MessageId) -> &Self {
        let mut inner = get_inner!(self.inner);
        inner.target_message = Some(msg);

        self
    }

    pub fn set_limit(&self, limit: usize) -> &Self {
        let mut inner = get_inner!(self.inner);
        inner.limit = limit;

        self
    }

    pub fn start_collecting<T: From<CollectorValue> + Collectible + Clone + 'static>(
        self,
    ) -> Receiver<T> {
        let inner = self.inner.clone();
        let mut inner = inner
            .lock()
            .expect("Error locking inner in CustomCollector::start_collecting");

        let (sender, receiver) = channel(inner.limit);

        let sender = Arc::new(Mutex::new(sender));

        let inner_collector = inner.collector.clone();
        let mut inner_collector = inner_collector.lock().expect("Error getting collector");

        let self_arc = Arc::new(self);

        let id = inner_collector
            .add_listener_value(T::collector_type(), move |value: Option<CollectorValue>| {
                let self_arc = self_arc.clone();

                let inner = self_arc.inner.clone();
                let mut inner = inner
                    .lock()
                    .expect("Error locking inner in CustomCollector::start_collecting");

                inner.count += 1;

                let sender = sender.clone();
                let mut sender = sender
                    .lock()
                    .expect("Error locking owned sender in CustomCollector::start_collecting");

                let value = value
                    .expect("Invariant: Listener did not emit a value for CollectorItem. Fatal.");

                let correct_channel = inner
                    .target_channel
                    .map(|channel| channel == value.channel_id())
                    .unwrap_or(true);

                let correct_user = inner
                    .target_user
                    .map(|user| value.author_id() == user)
                    .unwrap_or(true);

                let correct_message = inner
                    .target_message
                    .map(|msg| value.message_id() == msg)
                    .unwrap_or(true);

                // if you remove the (+ 1) the emitter doesn't collect enough messages so dont do that thanks
                if inner.count > inner.limit + 1 {
                    let self_arc = self_arc.clone();
                    let inner = self_arc.inner.clone();

                    thread::spawn(move || {
                        let inner = inner.lock().unwrap();

                        let mut collector = inner.collector.lock().expect(
                            "Error locking collector in ParallelEventEmitter::add_listener_value",
                        );

                        match collector.remove_listener(T::collector_type(), inner.listener_id.unwrap()) {
                            Ok(true) => (),
                            Ok(false) => panic!("Listener Invariant. remove_listener removed a listener that was already removed"),
                            Err(e) => panic!("Error removing listener {}", e),
                        }
                    });
                } else if correct_channel && correct_user && correct_message {
                    sender.try_send(T::from(value)).expect("Error sending message to owned channel in ParallelEventEmitter::add_listener_value")
                }

                Ok(())
            })
            .expect("Error while adding event listener to collector.");

        inner.listener_id = Some(id);

        receiver
    }
}
