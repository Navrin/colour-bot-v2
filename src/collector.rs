use std::sync::{Arc, Mutex, MutexGuard};

use serenity::model::channel::Message;
use serenity::model::id::{ChannelId, UserId};

use futures::prelude::*;
use futures::sync::mpsc::{channel, Receiver};

use parallel_event_emitter::{ListenerId, ParallelEventEmitter};

#[derive(Hash, Clone, Eq, PartialEq)]
pub enum CollectorItem {
    Message,
    Reaction,
}

pub struct Collector(pub Mutex<ParallelEventEmitter<CollectorItem>>);

impl Collector {
    pub fn new() -> Self {
        Collector(Mutex::new(ParallelEventEmitter::new()))
    }

    pub fn begin_blocking_collect<CB>(&self, filter: CB, limit: usize) -> Receiver<Message>
    where
        CB: Fn(&Message) -> bool + 'static,
    {
        let (sender, receiver) = channel(limit);
        let sender = Arc::new(Mutex::new(sender));

        let mut inner = self.0.lock().unwrap();

        println!("Collecting testtest");

        let x = inner
            .add_listener_value(CollectorItem::Message, move |msg| {
                let sender = sender.clone();

                println!("{}", "hewwo");
                let msg = msg.unwrap();

                if filter(&msg) {
                    let mut sender = sender.lock().expect(
                        "Something broke while locking an internal channel in the collect function",
                    );
                    sender.try_send(msg);
                }

                Ok(())
            })
            .unwrap();

        println!("{}", x);

        receiver
    }
}

struct InnerCustomCollector {
    collector: Arc<Collector>,
    listener_id: Option<ListenerId>,
    target_channel: Option<ChannelId>,
    target_user: Option<UserId>,
    limit: usize,
    // count towards the limit so we know when to disconnect the listener.
    count: usize,
}

struct CustomCollector {
    inner: Arc<Mutex<InnerCustomCollector>>,
}

impl CustomCollector {
    pub fn new(collector: Arc<Collector>) -> Self {
        CustomCollector {
            inner: Arc::new(Mutex::new(InnerCustomCollector {
                collector,
                listener_id: None,
                target_channel: None,
                target_user: None,
                limit: 1,
                count: 0,
            })),
        }
    }

    fn get_inner(&self) -> MutexGuard<InnerCustomCollector> {
        let inner = self.inner.clone();
        inner
            .lock()
            .expect("Error locking inner for CustomCollector::get_inner")
    }

    /// Collector will only get messages from this channel.
    pub fn set_channel(&self, chan: ChannelId) -> &Self {
        let mut inner = self.get_inner();
        inner.target_channel = Some(chan);

        self
    }

    /// Collector will only get messages form this user.
    pub fn set_user(&self, user: UserId) -> &Self {
        let mut inner = self.get_inner();
        inner.target_user = Some(user);

        self
    }

    pub fn set_limit(&self, limit: usize) -> &Self {
        let mut inner = self.get_inner();
        inner.limit = limit;

        self
    }

    pub fn start_collecting(self) -> Receiver<Message> {
        let mut inner = self.get_inner();

        let (sender, receiver) = channel(inner.limit);

        let sender = Arc::new(Mutex::new(sender));

        let mut inner_collector = inner.collector.0.lock().expect("Error getting collector");

        let id = inner_collector
            .add_listener_value(CollectorItem::Message, move |message: Option<Message>| {
                let mut inner = self.get_inner();

                inner.count += 1;

                let sender = sender.clone();
                let mut sender = sender
                    .lock()
                    .expect("Error locking owned sender in CustomCollector::start_collecting");

                let message = message.expect(
                    "Invariant: Listener did not emit a value for CollectorItem::Message. Fatal.",
                );

                let correct_channel = inner
                    .target_channel
                    .map(|channel| channel == message.channel_id)
                    .unwrap_or(true);

                let correct_user = inner
                    .target_user
                    .map(|user| message.author.id == user)
                    .unwrap_or(true);

                if inner.count > inner.limit {
                } else if correct_channel && correct_user {
                    sender.try_send(message.clone());
                }

                Ok(())
            })
            .expect("Error while adding event listener to collector.");

        let mut inner = self.get_inner();
        inner.listener_id = Some(id);

        receiver
    }
}

// #[derive(Clone)]
// pub struct Collector(
//     Vec<
//         (
//             Sender<Message>,
//             Arc<Fn(&Message) -> bool + Send + Sync + 'static>,
//         ),
//     >,
// );

// impl Key for Collector {
//     type Value = Collector;
// }

// impl Collector {
//     pub fn new() -> Self {
//         Collector(Vec::new())
//     }

//     pub fn tick(&mut self, msg: Message) -> Result<(), SendError<Message>> {
//         let inner_vec = &mut self.0;

//         let len = inner_vec.len();

//         let iterable = inner_vec.iter_mut().zip(0..len);

//         for (&mut (ref mut sender, ref mut filter), index) in iterable {
//             if filter(&msg) {
//                 println!("{}", "ih");
//                 sender.send(msg.clone())?
//             }
//         }

//         Ok(())
//     }

//     fn make_recv<CB>(&mut self, filter: CB) -> Receiver<Message>
//     where
//         CB: Fn(&Message) -> bool + Send + Sync + 'static,
//     {
//         let (sender, receiver) = unbounded();

//         self.0.push((sender, Arc::new(filter)));

//         receiver
//     }

//     /// !!HEY!! you need to execute this on a **NEW** thread.
//     /// !! If you do not spawn a new thread, this will block and KILL the bot.
//     pub fn very_blocking_collect_messages<CB>(&mut self, filter: CB, limit: usize) -> Vec<Message>
//     where
//         CB: Fn(&Message) -> bool + Send + Sync + 'static,
//     {
//         let recv = self.make_recv(filter);
//         println!("{}", "recv");

//         recv.iter().take(limit).collect::<Vec<_>>()
//     }
// }
