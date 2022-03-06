use {
    rustc_hash::FxHashMap,
    neoproto::prelude::*,

    tokio::{
        sync::{
            mpsc::UnboundedSender,
        }
    },

    crate::{
        ipc::*,
    }
};

pub struct Channels {
    pub clients: FxHashMap<ClientId, UnboundedSender<SlaveFrame>>,
}

impl Channels {
    #[inline]
    pub fn add_client(
        &mut self,
        id: ClientId,
        tx: UnboundedSender<SlaveFrame>
    ) {
        self.clients.insert(id, tx);
    }

    pub fn send(
        &mut self,
        id: ClientId,
        frame: SlaveFrame,
    ) -> bool {
        match self.clients.get(&id) {
            Some(tx) => {
                tx.send(frame).is_ok()
            },

            None => false,
        }
    }

    #[inline]
    pub fn remove_client(
        &mut self,
        id: ClientId
    ) {
        self.clients.remove(&id);
    }

    pub fn new() -> Self {
        Self{
            clients: Default::default()
        }
    }
}
