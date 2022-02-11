pub type PacketType = u8;
pub type ClientId = u16;
pub type PacketLength = u16;
pub type Flags = u8;

pub mod protocol {
    // pkt_type flags
    pub const SHORT: u8 = 0b001; // Short packet(payload length <= 255)
    pub const C_SHORT: u8 = 0b010; // Short client id(1 byte)
    pub const COMPRESSED: u8 = 0b100; // Compression flag

    // Packet types
    pub const ERROR: u8 = 0;
    pub const SERVER: u8 = 1;
    pub const PING: u8 = 2;

    pub const SERVER_SHUTDOWN: u8 = 3;

    pub const CONNECT: u8 = 4;
    pub const PACKET: u8 = 5;
    pub const DISCONNECT: u8 = 6;

    pub const AUTH_MAGIC: u8 = 7; // Auth through magic
    pub const REJECT: u8 = 8; // reject all rights

    pub const RIGHTS_UPDATE: u8 = 9;

    pub const SYNCHRONIZE: u8 = 10;  // Sync server settings with the client
                                     // e.g. compression level

    // Errors
    pub const NO_SUCH_COMMAND: u8 = 0;
    pub const NO_SUCH_CLIENT: u8 = 1;
    pub const NO_SESSION: u8 = 2;
    pub const ALREADY_CREATED: u8 = 3;

    pub const TOO_LONG_BUFFER: u8 = 4;
    pub const DECOMPRESSION_ERROR: u8 = 5;

    pub const ACCESS_DENIED: u8 = 6;

    pub const CANT_BIND_PORT: u8 = 7;
    pub const PROXY_DOWN: u8     = 8;
}

#[inline(always)]
pub fn error_to_string(
    code: u8
) -> &'static str {
    match code {
        protocol::NO_SUCH_COMMAND     => "No such command",
        protocol::NO_SUCH_CLIENT      => "No such client (invalid ClientId)",

        protocol::NO_SESSION          => "Server is not created yet",
        protocol::ALREADY_CREATED     => "Server is already created",

        protocol::TOO_LONG_BUFFER     => "Too long buffer size",
        protocol::DECOMPRESSION_ERROR => "Buffer decompression error",

        protocol::ACCESS_DENIED       => "Access denied",
        protocol::CANT_BIND_PORT      => "Can't bind specified port",

        _                             => "Unknown error",
    }
}

#[inline(always)]
pub fn serialize_variadic(
    dest: &mut [u8],
    value: u16
) -> bool {
    if value <= 0xff {
        dest[0] = value as u8;
        true
    } else {
        dest[0] = (value & 0xff) as u8;
        dest[1] = (value >> 8) as u8;
        false
    }
}

#[inline(always)]
pub fn unpack_type(
    data: u8
) -> (PacketType, Flags) {
    (data >> 3, data & 0b111)
}

#[inline(always)]
pub fn pack_type(
    type_: PacketType,
    flags: Flags
) -> u8 {
    (type_ << 3) | flags
}
