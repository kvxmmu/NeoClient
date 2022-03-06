use crate::channels::Channels;
use std::time::Duration;

pub struct Session {
    pub magic: Option<String>,

    pub compression_profit: f32,
    pub compression_level: i32,

    pub request_port: u16,

    pub channels: Channels,
    pub timeout: Duration,

    m_sent: bool,
}

impl Session {
    pub fn new(
        magic: Option<String>,

        compression_profit: f32,
        compression_level: i32,

        request_port: u16,
        timeout: Duration,
    ) -> Self {
        Self{
            magic,

            compression_level,
            compression_profit,

            request_port,

            m_sent: false,
            
            channels: Channels::new(),
            timeout,
        }
    }

    #[inline]
    pub fn make_sent(&mut self) -> bool {
        if self.m_sent {
            false
        } else {
            self.m_sent = true;
            true
        }
    }
}
