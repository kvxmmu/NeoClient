#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecSide {
    Server,
    Client,
}

impl CodecSide {
    #[inline(always)]
    pub fn is_client(self) -> bool {
        self == Self::Client
    }

    #[inline(always)]
    pub fn is_server(self) -> bool {
        self == Self::Server
    }
}
