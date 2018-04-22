use serenity::model::prelude::Message;
use std::ops::{Deref, DerefMut};
use std::thread::{self, sleep, spawn};
use std::time::Duration;

pub struct DeleteOnDrop {
    pub message: Message,
    delay: u64,
}

impl DeleteOnDrop {
    pub fn new(message: Message, delay: u64) -> Self {
        DeleteOnDrop { message, delay }
    }
}

impl Drop for DeleteOnDrop {
    fn drop(&mut self) {
        println!("dropping");

        let msg = self.message.clone();
        let del = self.delay;
        thread::spawn(move || {
            sleep(Duration::from_secs(del));

            msg.delete();
        });
    }
}

impl Deref for DeleteOnDrop {
    type Target = Message;

    fn deref(&self) -> &Self::Target {
        &self.message
    }
}

impl DerefMut for DeleteOnDrop {
    fn deref_mut(&mut self) -> &mut Message {
        &mut self.message
    }
}
