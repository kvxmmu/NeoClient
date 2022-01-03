pub type PacketType   = u8;
pub type ClientId     = u16;
pub type PacketLength = u16;
pub type Flags        = u8;

pub mod protocol {
    // pkt_type flags
    pub const SHORT: u8      = 0b001;  // Short packet(payload length <= 255)
    pub const C_SHORT: u8    = 0b010;  // Short client id(1 byte)
    pub const COMPRESSED: u8 = 0b100;  // Compression flag

    // Packet types
    pub const ERROR: u8    = 0;
    pub const SERVER: u8   = 1;
    pub const PING: u8     = 2;

    pub const SERVER_SHUTDOWN: u8 = 3;

    pub const CONNECT: u8    = 4;
    pub const PACKET: u8     = 5;
    pub const DISCONNECT: u8 = 6;

    pub const AUTH_MAGIC: u8 = 7;  // Auth through magic
    pub const REJECT: u8     = 8;  // reject all rights

    pub const RIGHTS_UPDATE: u8 = 9;

    // Errors
    pub const NO_SUCH_COMMAND: u8     = 0;
    pub const NO_SUCH_CLIENT: u8      = 1;
    pub const NO_SESSION: u8          = 2;
    pub const ALREADY_CREATED: u8     = 3;

    pub const TOO_LONG_BUFFER: u8     = 4;
    pub const DECOMPRESSION_ERROR: u8 = 5;

    pub const ACCESS_DENIED: u8       = 6;

    pub const CANT_BIND_PORT: u8      = 7;
}

pub mod right {
    pub const CREATE_SERVER: u8 = 0b01;
    pub const SELECT_PORT: u8   = 0b10;

    pub const ALL: u8 = CREATE_SERVER | SELECT_PORT;

    pub fn to_string(rights: u8) -> String {
        let mut buf = String::new();

        if (rights & CREATE_SERVER) != 0 {
            buf.push_str(" | CREATE_SERVER");
        }
        if (rights & SELECT_PORT) != 0 {
            buf.push_str(" | SELECT_PORT");
        }

        String::from(&buf[3..])
    }
}

#[inline(always)]
pub fn serialize_variadic(dest: &mut [u8], value: u16) -> bool {
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
pub fn unpack_type(data: u8) -> (PacketType, Flags) {
    (data >> 3, data & 0b111)
}

#[inline(always)]
pub fn pack_type(type_: PacketType, flags: Flags) -> u8 {
    (type_ << 3) | flags
}

