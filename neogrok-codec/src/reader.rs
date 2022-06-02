use {
    tokio::{
        io::{ AsyncReadExt }
    },

    protocol::prelude::*,
    std::{ io
         , marker::Unpin
         , future::Future
         , intrinsics::unlikely },
    
    crate::{ unsafe_buffer::*
           , side::CodecSide },
    zstd_wrapper::dctx::ZStdDctx,
};

pub struct CodecReader<OwnedHalf> {
    inner: OwnedHalf,
    dctx: ZStdDctx,

    pub max_payload_size: usize,
    pub side: CodecSide,
}

impl<OwnedHalf: AsyncReadExt + Unpin> CodecReader<OwnedHalf> {
    pub async fn read_frame(
        &mut self
    ) -> io::Result<NFrame> {
        let pkt_data = self.read_type().await?;
        let pkt_type = remove_flags(pkt_data);
        let max_payload_size = self.max_payload_size;

        let frame = match pkt_type {
            PING if self.side.is_server() => NFrame::Ping,
            PING => NFrame::PingResponse {
                name: self.read_string().await?,
            },

            AUTH if self.side.is_server()  => {
                let length = self.inner.read_u8().await? as usize;
                let mut buffer = unsafe { create_buffer(length) };

                self.inner.read_exact(&mut buffer).await?;
                
                NFrame::Authorize { magic: buffer }
            },

            SYNC if self.side.is_server() => NFrame::Sync,
            SYNC => NFrame::SyncResponse {
                level: self.inner.read_u8().await? as u16,
                profit: self.inner.read_f32_le().await?,
                threshold: self.inner.read_u16_le().await?,
            },

            SERVER => {
                if pkt_data.is_short() {
                    NFrame::Server {
                        protocol: TransportProtocol::Tcp,
                        port: 0
                    }
                } else if pkt_data.is_c_short() {
                    NFrame::Server {
                        protocol: TransportProtocol::Tcp,
                        port: self.inner.read_u16_le().await?
                    }
                } else if pkt_data.is_compressed() {
                    NFrame::Server {
                        protocol: TransportProtocol::from(self.inner.read_u8().await?),
                        port: 0
                    }
                } else {
                    NFrame::Server {
                        protocol: TransportProtocol::from(self.inner.read_u8().await?),
                        port: self.inner.read_u16_le().await?,
                    }
                }
            },

            CONNECT => {
                NFrame::Connect { id: self.read_client_id(pkt_data).await? }
            },

            DISCONNECT => {
                NFrame::Disconnect { id: self.read_client_id(pkt_data).await? }
            },

            FORWARD => {
                let id = self.read_client_id(pkt_data).await?;
                let length = self.read_length(pkt_data).await? as usize;

                if unlikely(length > self.max_payload_size) {
                    return Ok(NFrame::Error { code: TOO_LONG });
                }

                let mut buf = unsafe { create_buffer(length) };
                self.inner.read_exact(&mut buf).await?;

                if pkt_data.is_compressed() {
                    if ZStdDctx::decompressed_size_of(&buf).map(|r| r.get() > max_payload_size).unwrap_or(true) {
                        return Ok(NFrame::Error { code: TOO_LONG });
                    }

                    let decompressed = self.dctx.decompress_async(&mut buf).await;
                    if let Some(buf) = decompressed {
                        NFrame::Forward { id, buffer: buf }
                    } else {
                        NFrame::Error { code: DECOMPRESS_ERR }
                    }
                } else {
                    NFrame::Forward { id, buffer: buf }
                }
            },

            ERROR => NFrame::Error { code: self.inner.read_u8().await? },

            _ => NFrame::Error { code: UNKNOWN_PKT }
        };

        Ok(frame)
    }

    pub fn read_client_id(
        &mut self,
        flags: u8
    ) -> impl Future<Output = io::Result<u16>> + '_ {
        self.read_variadic(
            flags,
            C_SHORT
        )
    }

    pub async fn read_string(
        &mut self,
    ) -> io::Result<String> {
        let length = self.inner.read_u8().await? as usize;
        let mut buf = unsafe { create_buffer(length) };

        self.inner.read_exact(&mut buf).await?;
        Ok(String::from_utf8_lossy(&buf)
                 .to_string())
    }

    pub fn read_length(
        &mut self,
        flags: u8
    ) -> impl Future<Output = io::Result<u16>> + '_ {
        self.read_variadic(
            flags,
            SHORT
        )
    }

    pub async fn read_variadic(
        &mut self,
        flags: u8,
        need: u8
    ) -> io::Result<u16> {
        if (flags & need) == need {
            Ok(self.inner.read_u8().await? as u16)
        } else {
            self.inner.read_u16_le().await
        }
    }

    #[inline(always)]
    pub fn read_type(
        &mut self
    ) -> impl Future<Output = io::Result<u8>> + '_ {
        self.inner.read_u8()
    }

    pub fn client(
        inner: OwnedHalf
    ) -> Self {
        Self::new(inner, usize::MAX, CodecSide::Client)
    }

    pub fn server(
        inner: OwnedHalf,
        max_payload_size: usize
    ) -> Self {
        Self::new(inner, max_payload_size, CodecSide::Server)
    }

    pub fn new(
        inner: OwnedHalf,
        max_payload_size: usize,
        side: CodecSide,
    ) -> Self {
        Self { inner
             , dctx: ZStdDctx::new()
             , max_payload_size
             , side }
    }
}
