use {
    neoproto::prelude::*,
    crate::ipc::*,
};

#[derive(Debug)]
pub enum Frame {
    Error(u8),
    CreatedTCP(u16),
    Pong(String),

    Connected(ClientId),
    Disconnected(ClientId),

    Packet(ClientId, Vec<u8>, usize),

    UpdateRights(u8),
    Synchronize(u8, u8, i32),  // initial_rights, magic_rights, compression_level

    UnknownCommand(u8, u8),

    HandleSlave(MasterFrame),
}
