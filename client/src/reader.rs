use {
    tokio::{
        net::TcpStream,
        io::AsyncReadExt,
    },

    crate::{
        frame::*,
        compressor::*,
        net,
    },

    neoproto::prelude::*,
};


pub async fn read_frame(
    type_: u8,
    flags: u8,
    stream: &mut TcpStream,
) -> std::io::Result<Frame> {
    let result = match type_ {
        protocol::ERROR         => {
            Frame::Error(stream.read_u8().await?)
        },

        protocol::RIGHTS_UPDATE => {
            Frame::UpdateRights(stream.read_u8().await?)
        },

        protocol::CONNECT       => {
            Frame::Connected(
                net::read_variadic(stream, flags, protocol::C_SHORT).await?
            )
        },

        protocol::PING          => {
            Frame::Pong(
                net::read_string(stream).await?
            )
        },

        protocol::SYNCHRONIZE   => {
            let initial_rights = stream.read_u8().await?;
            let magic_rights = stream.read_u8().await?;
            let compression_level = stream.read_u8().await? as i32;

            Frame::Synchronize(initial_rights, magic_rights, compression_level)
        },

        protocol::PACKET        => {
            let mut length = net::read_variadic(stream, flags, protocol::SHORT).await? as usize;
            let id = net::read_variadic(stream, flags, protocol::C_SHORT).await?;
            let mut buf = vec![0; length];
            stream.read_exact(&mut buf).await?;

            if (flags & protocol::COMPRESSED) != 0 {
                let result = tokio::task::spawn_blocking(move || {
                    decompress(
                        buf, length << 2
                    )
                }).await?;
                if result.is_none() {
                    return Err(std::io::Error::last_os_error());
                }

                let (new_buf, new_length) = result.unwrap();
                length = new_length;
                buf = new_buf;
            }

            Frame::Packet(
                id, buf, length
            )
        },

        protocol::DISCONNECT    => {
            Frame::Disconnected(
                net::read_variadic(stream, flags, protocol::C_SHORT).await?
            )
        },

        protocol::SERVER        => {
            Frame::CreatedTCP(
                stream.read_u16_le().await?
            )
        },

        _ => Frame::UnknownCommand(type_, flags),
    };

    Ok(result)
}
