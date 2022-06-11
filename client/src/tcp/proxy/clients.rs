use {
    super::{
        frame::*,
    },
    
    rustc_hash::FxHashMap,
    idpool::*,

    tokio::{
        sync::mpsc::{ UnboundedSender }
    }
};

pub enum SendErrorKind {
    Closed,
    NoSuchClient,

    NoError,
}

pub struct ProxyClients {
    clients: FxHashMap<ClientId, UnboundedSender<SlaveFrame>>
}

impl ProxyClients {
    pub fn disconnect(
        &mut self,
        id: ClientId
    ) -> SendErrorKind {
        if let Some(sink) = self.clients.get(&id) {
            sink.send(SlaveFrame::ForceDisconnect)
                .unwrap_or_default();
            SendErrorKind::NoError
        } else {
            SendErrorKind::NoSuchClient
        }
    }

    pub fn forward(
        &self,
        id: ClientId,
        buffer: Vec<u8>,
    ) -> SendErrorKind {
        self.send(
            id,
            SlaveFrame::Forward { buffer }
        )
    }

    pub fn connected(
        &mut self,
        id: ClientId,
        sink: UnboundedSender<SlaveFrame>
    ) {
        self.clients
            .insert(id, sink);
    }

    pub fn send(
        &self,
        id: ClientId,
        frame: SlaveFrame
    ) -> SendErrorKind {
        if let Some(sink) = self.clients.get(&id) {
            if sink.send(frame).is_ok() {
                SendErrorKind::NoError
            } else {
                SendErrorKind::Closed
            }
        } else {
            SendErrorKind::NoSuchClient
        }
    }

    pub fn new() -> Self {
        Self { clients: Default::default() }
    }
}
