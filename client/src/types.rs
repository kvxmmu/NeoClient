use neoproto::prelude::*;

pub enum ConnectorRequest {
    Forward(Vec<u8>),
    Disconnect
}

pub enum MainRequest {
    Forward(ClientId, Vec<u8>, Flags),
    Disconnect(ClientId),

    EndOfLife(ClientId)
}
