use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use serenity::model::id::MessageId;
use typemap::Key;

pub struct Cleaner(pub HashSet<MessageId>);

impl Cleaner {
    pub fn new() -> Self {
        Cleaner(HashSet::new())
    }
}

impl Key for Cleaner {
    type Value = Cleaner;
}

impl Deref for Cleaner {
    type Target = HashSet<MessageId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Cleaner {
    fn deref_mut(&mut self) -> &mut HashSet<MessageId> {
        &mut self.0
    }
}
