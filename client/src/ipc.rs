use {
    neoproto::prelude::*,
};

#[derive(Debug)]
pub enum MasterFrame {
    Forward(ClientId, Vec<u8>),
    Disconnected(ClientId),
}

#[derive(Debug)]
pub enum SlaveFrame {
    Forward(Vec<u8>, usize),
    ForceDisconnect,
}
