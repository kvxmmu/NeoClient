use {
    tokio::{
        net::TcpStream,
        io::{AsyncWriteExt, AsyncReadExt}
    },

    std::{
        future::Future,
    },

    crate::compressor::compress,
    neoproto::prelude::*,
};

const SKIP_CHUNK: usize = 128;

/* Read variadic type which size is determined by flags */
pub async fn read_variadic(
    stream: &mut TcpStream,
    flags: u8,
    need: u8,
) -> std::io::Result<u16> {
    if (flags & need) == need {
        Ok(stream.read_u8().await? as u16)
    } else {
        stream.read_u16_le().await
    }
}

pub async fn send_sync_info(
    stream: &mut TcpStream,
) -> std::io::Result<()> {
    stream.write_u8(
        pack_type(protocol::SYNCHRONIZE, 0),
    ).await
}

pub async fn send_magic_auth(
    stream: &mut TcpStream,
    magic: &str
) -> std::io::Result<()> {
    stream.write_all(&[
        pack_type(protocol::AUTH_MAGIC, 0),
        magic.len() as u8,
    ]).await?;
    stream.write_all(magic.as_bytes()).await
}

pub async fn send_ping(
    stream: &mut TcpStream,
) -> std::io::Result<()> {
    stream.write_u8(
        pack_type(protocol::PING, 0),
    ).await
}

/* Read string with unsigned 8bit length */
pub async fn read_string(
    stream: &mut TcpStream,
) -> std::io::Result<String> {
    let length = stream.read_u8().await? as usize;
    let mut buf = vec![0; length];

    stream.read_exact(&mut buf).await?;
    match String::from_utf8(buf) {
        Ok(r) => Ok(r),
        _ => Err(std::io::Error::last_os_error())
    }
}

/* Skip n bytes to reduce some client errors */
pub async fn skip_n_bytes(
    stream: &mut TcpStream,
    n: usize,
) -> std::io::Result<()> {
    let mut buf = [0; SKIP_CHUNK];
    let mut received = 0;

    while received < n {
        let mut diff = n - received;
        if diff > SKIP_CHUNK {
            diff = SKIP_CHUNK;
        }

        stream.read_exact(&mut buf[..diff]).await?;
        received += diff;
    }

    Ok(())
}

/* Read packed packet type */
pub async fn read_type(
    stream: &mut TcpStream,
) -> std::io::Result<(PacketType, Flags)> {
    let mut target = [0; 1];
    let result = stream.read(&mut target).await;
    match result {
        Ok(0) | Err(_) => {
            return Err(std::io::Error::last_os_error());
        },

        _ => Ok(unpack_type(target[0]))
    }
}

#[inline(always)]
pub fn send_connect(
    stream: &mut TcpStream,
    id: ClientId,
) -> impl Future<Output = std::io::Result<()>> + '_ {
    send_client_id(
        stream,
        protocol::CONNECT,
        id
    )
}

#[inline(always)]
pub fn send_disconnect(
    stream: &mut TcpStream,
    id: ClientId,
) -> impl Future<Output = std::io::Result<()>> + '_ {
    send_client_id(
        stream,
        protocol::DISCONNECT,
        id
    )
}

pub async fn send_client_id(
    stream: &mut TcpStream,
    type_: u8,
    id: ClientId,
) -> std::io::Result<()> {
    if id <= 0xff {
        stream.write_all(&[
            pack_type(type_, protocol::C_SHORT),
            id as u8,
        ]).await
    } else {
        stream.write_all(&[
            pack_type(type_, 0),
            (id & 0xff) as u8,
            (id >> 8) as u8,
        ]).await
    }
}

async fn forward_header(
    stream: &mut TcpStream,
    id: ClientId,
    length: u16,
    mut flags: u8,
) -> std::io::Result<()> {
    let mut buf = [0; 5];
    let mut offset = 1;

    offset += if serialize_variadic(&mut buf[offset..], length) {
        flags |= protocol::SHORT;
        1
    } else {
        2
    };

    offset += if serialize_variadic(&mut buf[offset..], id) {
        flags |= protocol::C_SHORT;
        1
    } else {
        2
    };

    buf[0] = pack_type(protocol::PACKET, flags);
    stream.write_all(&buf[..offset]).await
}

pub async fn forward_packet(
    stream: &mut TcpStream,
    id: ClientId,
    length: u16,

    buf: Vec<u8>,

    min_profit: f32,
    level: i32,
) -> std::io::Result<()> {
    let (result, buf) = tokio::task::spawn_blocking(move || {
        (
            compress(
                &buf[..length as usize],
                min_profit,
                level
            ),
            buf
        )
    }).await?;

    if let Some((new_buf, new_length)) = result {
        forward_header(stream, id, new_length as u16, protocol::COMPRESSED).await?;
        stream.write_all(&new_buf[..new_length]).await
    } else {
        forward_header(stream, id, length, 0).await?;
        stream.write_all(&buf).await
    }
}

pub async fn send_port(
    stream: &mut TcpStream,
    port: u16
) -> std::io::Result<()> {
    stream.write_all(&[
        pack_type(protocol::SERVER, 0),
        (port & 0xff) as u8,
        (port >> 8) as u8
    ]).await
}

pub async fn send_update_rights(
    stream: &mut TcpStream,
    rights: u8,
) -> std::io::Result<()> {
    stream.write_all(&[
        pack_type(protocol::RIGHTS_UPDATE, 0),
        rights
    ]).await
}

pub async fn send_error(
    stream: &mut TcpStream,
    code: u8,
) -> std::io::Result<()> {
    stream.write_all(&[
        pack_type(protocol::ERROR, 0),
        code
    ]).await
}

pub fn ok_or_exit<T>(
    r: std::io::Result<T>
) -> T {
    match r {
        Err(e) => {
            log::error!("NeoGrok error: {}", e);
            std::process::exit(1);
        },

        Ok(r) => r
    }
}
