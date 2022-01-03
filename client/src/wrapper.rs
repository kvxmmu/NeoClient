use {
    tokio::{
        net::TcpStream,
        io::{AsyncReadExt, AsyncWriteExt}
    },
    neoproto::prelude::*
};

pub fn ok_or_exit<T>(result: std::io::Result<T>) -> T {
    match result {
        Ok(r) => r,
        Err(e) => {
            println!("<< NeoGrok error: {}", e.to_string());
            std::process::exit(0);
        }
    }
}

#[async_trait::async_trait]
pub trait ProtocolExt {
    async fn read_type(&mut self) -> (PacketType, Flags);
    async fn write_simple(&mut self, type_: PacketType, buf: Option<&[u8]>,
                          flags: Flags);

    async fn read_variadic(&mut self, flags: Flags, need: Flags) -> u16;
    async fn authorize(&mut self, magic: &str);

    async fn read_port(&mut self) -> u16;
    async fn read_string(&mut self) -> String;

    async fn read_byte(&mut self) -> u8;

    async fn write_packet_header(&mut self, client_id: ClientId, length: PacketLength,
                                 flags: Flags);
    
    async fn write_client_id(&mut self, type_: PacketType, client_id: ClientId) {
        let mut buf: [u8; 2] = [0; 2];
        let flags;

        let offset = if serialize_variadic(&mut buf, client_id) {
            flags = protocol::C_SHORT;
            1
        } else { flags = 0; 2 };

        self.write_simple(type_, Some(&buf[..offset]), flags).await
    }

    async fn write_connect(&mut self, client_id: ClientId) {
        self.write_client_id(protocol::CONNECT, client_id).await
    }

    async fn write_disconnect(&mut self, client_id: ClientId) {
        self.write_client_id(protocol::DISCONNECT, client_id).await
    }

    async fn read_rights(&mut self) -> u8 {
        self.read_byte().await
    }

    async fn read_error(&mut self) -> u8 {
        self.read_byte().await
    }

    async fn request_server(&mut self, port: u16) {
        let buf = [(port & 0xff) as u8, (port >> 8) as u8];

        self.write_simple(protocol::SERVER, Some(&buf), 0).await;
    }

    async fn read_client_id(&mut self, flags: Flags) -> ClientId {
        self.read_variadic(flags, protocol::C_SHORT).await
    }

    async fn read_length(&mut self, flags: Flags) -> PacketLength {
        self.read_variadic(flags, protocol::SHORT).await
    }

    async fn request_ping(&mut self) {
        self.write_simple(protocol::PING, None, 0).await;
    }
}

#[async_trait::async_trait]
impl ProtocolExt for TcpStream {
    async fn read_type(&mut self) -> (PacketType, Flags) {
        unpack_type(ok_or_exit(self.read_u8().await))
    }

    async fn write_simple(&mut self, type_: PacketType, buf: Option<&[u8]>, flags: Flags) {
        ok_or_exit(self.write_u8( pack_type(type_, flags) ).await);
        if buf.is_some() {
            ok_or_exit(self.write_all(buf.unwrap()).await);
        }
    }

    async fn read_variadic(&mut self, flags: Flags, need: Flags) -> u16 {
        if (flags & need) != need {
            ok_or_exit( self.read_u16_le().await )
        } else {
            ok_or_exit( self.read_u8().await ) as u16
        }
    }

    async fn authorize(&mut self, magic: &str) {
        let mut buf: [u8; 2] = [0; 2];
        buf[0] = pack_type(protocol::AUTH_MAGIC, 0);
        buf[1] = magic.len() as u8;

        ok_or_exit(self.write_all(&buf).await);
        ok_or_exit(self.write_all(magic.as_bytes()).await);
    }

    async fn read_port(&mut self) -> u16 {
        ok_or_exit( self.read_u16_le().await )
    }

    async fn read_string(&mut self) -> String {
        let length = ok_or_exit( self.read_u8().await ) as usize;
        let mut buf = vec![0; length];
        ok_or_exit( self.read_exact(&mut buf).await );

        match String::from_utf8(buf) {
            Ok(s) => s,

            Err(e) => {
                println!("<< Failed to parse utf8 string: {}", e.to_string());
                std::process::exit(1);
            }
        }
    }

    async fn read_byte(&mut self) -> u8 {
        ok_or_exit( self.read_u8().await )
    }

    async fn write_packet_header(&mut self, client_id: ClientId, length: PacketLength,
                                 mut flags: Flags) {
        let mut buf: [u8; 5] = [0; 5];
        let mut offset = 1usize;

        offset += if serialize_variadic(&mut buf[offset..], length) {
            flags |= protocol::SHORT;
            1
        } else {
            2
        };
        offset += if serialize_variadic(&mut buf[offset..], client_id) {
            flags |= protocol::C_SHORT;
            1
        } else {
            2
        };
        
        buf[0] = pack_type(protocol::PACKET, flags);
        
        ok_or_exit( self.write_all(&buf[..offset]).await );
    }
}

