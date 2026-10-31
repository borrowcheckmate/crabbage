use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use tokio::sync::broadcast;

#[derive(Clone, Default)]
pub(crate) struct ShoppingHub {
    senders: Arc<Mutex<HashMap<i64, broadcast::Sender<()>>>>,
}

impl ShoppingHub {
    pub(crate) fn publish(&self, household: i64) {
        let senders = self.senders.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(sender) = senders.get(&household) {
            drop(sender.send(()));
        }
    }

    pub(crate) fn subscribe(&self, household: i64) -> broadcast::Receiver<()> {
        let mut senders = self.senders.lock().unwrap_or_else(PoisonError::into_inner);
        senders
            .entry(household)
            .or_insert_with(|| broadcast::channel(32).0)
            .subscribe()
    }
}
