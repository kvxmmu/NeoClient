use {
    std::{
        collections::VecDeque,
        sync::{Arc, atomic::{AtomicU16, Ordering}},
    },

    tokio::sync::Mutex,
};

pub type ClientId = u16;
pub type AtomicClientId = AtomicU16;
pub type SharedIDPool = Arc<IDPool>;

#[derive(Debug)]
pub struct IDPool {
    deq: Mutex<VecDeque<ClientId>>,
    last: AtomicClientId,
}

impl IDPool {
    pub async fn request_id(&self) -> ClientId {
        if let Some(id) = self.deq.lock().await.pop_front() {
            id
        } else {
            self.last.fetch_add(1, Ordering::Relaxed)
        }
    }

    pub async fn return_id(&self, id: ClientId) {
        self.deq.lock()
                .await
                .push_back(id)
    }

    pub fn zero() -> Self {
        Self::new(0)
    }

    pub fn new(start: ClientId) -> Self {
        Self { deq: Default::default()
             , last: AtomicClientId::new(start) }
    }
}
