use {
    zstd_wrapper::cctx::ZStdCctx,

    tokio::{
        io::{ AsyncWriteExt }
    },

    std::{
        marker::Unpin,
        future::Future,
        io,
    },

    protocol::prelude::*,
};

pub struct CodecWriter<WriteHalf> {
    inner: WriteHalf,
    cctx: ZStdCctx,
    threshold: usize,
}

impl<WriteHalf: AsyncWriteExt + Unpin> CodecWriter<WriteHalf> {
    pub async fn write_ping_request(
        &mut self
    ) -> io::Result<()> {
        self.inner.write_u8(pack_type(PING, 0)).await?;
        self.flush()
            .await
    }

    pub async fn write_update_rights(
        &mut self,
        rights: u8
    ) -> io::Result<()> {
        self.inner.write_all(&[
            pack_type(UPDATE_RIGHTS, 0),
            rights
        ]).await?;
        self.flush().await
    }

    pub async fn write_ping_response(
        &mut self,
        name: &str
    ) -> io::Result<()> {
        self.inner.write_u8(pack_type(PING, 0)).await?;
        self.write_string(name).await?;
        self.flush().await
    }

    pub async fn write_forward(
        &mut self,
        id: u16,
        mut buf: Vec<u8>,
    ) -> io::Result<()> {
        let mut flags = 0;

        if buf.len() >= self.threshold {
            if let Some(buffer) = self.cctx.compress_async(&mut buf).await {
                buf = buffer;
                flags |= COMPRESSED;
            }
        }

        let length = buf.len() as u16;
        if id <= 0xff {
            flags |= C_SHORT;
        }
        if length <= 0xff {
            flags |= SHORT;
        }

        self.inner.write_u8(pack_type(FORWARD, flags)).await?;
        self.write_client_id(id).await?;
        self.write_length(length).await?;

        self.inner.write_all(&mut buf).await
    }

    pub async fn write_sync_request(
        &mut self
    ) -> io::Result<()> {
        self.inner.write_u8(pack_type(SYNC, 0)).await
    }

    pub async fn write_sync_response(
        &mut self,
        level: i32,
        profit: f32,
        threshold: u16
    ) -> io::Result<()> {
        self.inner.write_all(&[
            pack_type(SYNC, 0),
            level as u8,
        ]).await?;
        self.inner.write_f32(profit).await?;
        self.inner.write_u16_le(threshold).await
    }

    pub fn write_connect(
        &mut self,
        id: u16
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.write_pkt_client_id(
            CONNECT,
            id
        )
    }

    pub fn write_disconnect(
        &mut self,
        id: u16
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.write_pkt_client_id(
            DISCONNECT,
            id
        )
    }

    pub async fn write_pkt_client_id(
        &mut self,
        pkt_type: u8,
        id: u16
    ) -> io::Result<()> {
        self.inner.write_u8(pack_type(
            pkt_type,
            if id <= 0xff {
                C_SHORT
            } else {
                0
            }
        )).await?;
        self.write_variadic(
            C_SHORT,
            id
        ).await
         .map(|_| ())?;
        self.inner
            .flush()
            .await
    }

    pub async fn write_variadic(
        &mut self,
        flag: u8,
        value: u16
    ) -> io::Result<u8> {
        if value <= 0xff {
            self.inner.write_u8(value as u8).await?;
            Ok(flag)
        } else {
            self.inner.write_u16_le(value).await?;
            Ok(0)
        }
    }

    pub fn write_length(
        &mut self,
        value: u16
    ) -> impl Future<Output = io::Result<u8>> + '_ {
        self.write_variadic(
            SHORT,
            value
        )
    }

    pub fn write_client_id(
        &mut self,
        value: u16
    ) -> impl Future<Output = io::Result<u8>> + '_ {
        self.write_variadic(
            C_SHORT,
            value
        )
    }

    pub async fn write_error(
        &mut self,
        code: u8,
    ) -> io::Result<()> {
        self.inner.write_all(&[
            pack_type(ERROR, 0),
            code
        ]).await
    }

    pub async fn write_request_server(
        &mut self,
        protocol: TransportProtocol,
        port: u16
    ) -> io::Result<()> {
        match (protocol, port) {
            (TransportProtocol::Tcp, 0) => self.inner.write_u8(
                pack_type(SERVER, SHORT)
            ).await,

            (TransportProtocol::Tcp, port) => self.inner.write_all(&[
                pack_type(SERVER, C_SHORT),
                port as u8 & 0xff,
                port as u8 >> 8
            ]).await,

            (proto, 0) => self.inner.write_all(&[
                pack_type(SERVER, COMPRESSED),
                proto.into()
            ]).await,

            _ => self.inner.write_all(&[
                pack_type(SERVER, 0),
                protocol.into(),
                port as u8 & 0xff,
                port as u8 >> 8
            ]).await
        }
    }

    pub async fn write_port(
        &mut self,
        port: u16
    ) -> io::Result<()> {
        self.inner.write_all(&[
            pack_type(SERVER, 0),
            port as u8 & 0xff,
            port as u8 >> 8
        ]).await?;
        self.flush()
            .await
    }

    pub async fn write_string(
        &mut self,
        src: &str
    ) -> io::Result<()> {
        self.inner.write_u8(src.len() as u8).await?;
        self.inner.write_all(src.as_bytes()).await
    }

    pub fn flush(
        &mut self
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.inner.flush()
    }

    pub fn new(
        half: WriteHalf,

        level: i32,
        profit: f32,
        threshold: usize,
    ) -> Self {
        Self { inner: half
             , cctx: ZStdCctx::new(level, profit)
             , threshold }
    }
}
