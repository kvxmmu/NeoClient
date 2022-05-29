#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportProtocol {
    Tcp,
    Udp,
    Http,
}

#[derive(Debug, Clone)]
pub enum NFrame {
    Ping,
    Sync,

    Server {
        protocol: TransportProtocol,
        port: u16,
    },

    Authorize {
        magic: Vec<u8>,  // actually this is the string
    },

    Connect {
        id: u16,
    },

    Forward {
        id: u16,
        buffer: Vec<u8>,
    },

    Disconnect {
        id: u16
    },

    Error { code: u8 },

    /// Responses
    PingResponse {
        name: String,
    },

    
    ServerResponse {
        host: String,
        port: u16,
    },

    SyncResponse {
        threshold: u16,
        level: u16,
        profit: f32,
    }
}

impl Default for TransportProtocol {
    fn default() -> Self {
        Self::Tcp
    }
}

impl Into<u8> for TransportProtocol {
    fn into(self) -> u8 {
        match self {
            Self::Udp => 0,
            Self::Http => 1,
            _ => 2,
        }
    }
}

impl From<u8> for TransportProtocol {
    fn from(num: u8) -> Self {
        match num {
            0 => Self::Udp,
            1 => Self::Http,
            _ => Self::Tcp
        }
    }
}
