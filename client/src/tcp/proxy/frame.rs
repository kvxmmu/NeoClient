use idpool::ClientId;

pub enum MasterFrame {
    Forward {
        id: ClientId,
        buffer: Vec<u8>,
    },

    Disconnected {
        id: ClientId,
    }
}

pub enum SlaveFrame {
    Forward {
        buffer: Vec<u8>,
    },

    ForceDisconnect,
}
