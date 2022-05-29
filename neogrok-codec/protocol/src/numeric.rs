/// Packet types
pub const ERROR: u8            = 0;
pub const PING: u8             = 1;
pub const AUTH: u8             = 2;

pub const CONNECT: u8          = 3;
pub const FORWARD: u8          = 4;
pub const DISCONNECT: u8       = 5;

pub const SYNC: u8             = 6;

pub const SERVER: u8           = 7;

pub const UPDATE_RIGHTS: u8    = 8;

/// Error types
pub const UNIMPLEMENTED: u8    = 0;
pub const ACCESS_DENIED: u8    = 1;
pub const BIND_ERROR: u8       = 2;
pub const UNKNOWN_PKT: u8      = 3;
pub const UNSUPPORTED_PKT: u8  = 4;

pub const TOO_LONG: u8         = 5;
pub const DECOMPRESS_ERR: u8   = 6;

/// Packet flags
pub const COMPRESSED: u8       = 0b001;  // The packet is compressed
pub const SHORT: u8            = 0b010;  // The packet is short(e.g. length field size = 1byte)
pub const C_SHORT: u8          = 0b100;  // Client id field is short (length = 1byte)

/// Rights flags
pub const CAN_CREATE_HTTP: u8          = 0b00000001;
pub const CAN_CREATE_TCP: u8           = 0b00000010;
pub const CAN_CREATE_UDP: u8           = 0b00000100;
pub const CAN_CREATE_SSH: u8           = 0b00001000;

pub const CAN_SELECT_HTTP: u8          = 0b00010000;
pub const CAN_SELECT_UDP: u8           = 0b00100000;
pub const CAN_SELECT_TCP: u8           = 0b01000000;
