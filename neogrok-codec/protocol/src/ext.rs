use crate::numeric::*;

macro_rules! gen_flag_check {
    () => {};
    ($name:ident & $flag:expr$(, $($tail:tt)*)?) => {
        #[inline(always)]
        fn $name(self) -> bool {
            let flag = $flag;
            (self & $flag) == flag
        }

        gen_flag_check!($(
            $($tail)*
        )?);
    };
}

macro_rules! gen_flag_str_serializer {
    () => {};
    ($to:ident <- $lit:literal ? $e:expr$(, $($tail:tt)*)?) => {
        if $e {
            $to.push_str($lit);
            $to.push_str(" |");
        }

        gen_flag_str_serializer!($($($tail)*)?);
    };
}

impl TypeFlags for u8 {
    gen_flag_check! {
        is_short      & SHORT,
        is_c_short    & C_SHORT,
        is_compressed & COMPRESSED
    }
}

impl RightsFlags for u8 {
    gen_flag_check! {
        can_create_http      & CAN_CREATE_HTTP,
        can_create_tcp       & CAN_CREATE_TCP,
        can_create_udp       & CAN_CREATE_UDP,

        can_select_http_path & CAN_SELECT_HTTP,
        can_select_tcp_port  & CAN_SELECT_TCP,
        can_select_udp_port  & CAN_SELECT_UDP
    }
}

impl ErrorToString for u8 {
    fn to_error_string(self) -> &'static str {
        match self {
            ACCESS_DENIED => "Access Denied",
            UNIMPLEMENTED => "Currently this feature is not supported",
            BIND_ERROR => "Can't bind specified port (Possibly already bound)",
            UNKNOWN_PKT => "Unknown packet sent",
            UNSUPPORTED_PKT => "Unsupported packet type",
            TOO_LONG => "Too long packet buffer",

            _ => "Unknown error"
        }
    }
}

impl RightsToString for u8 {
    fn show_rights(self) -> String {
        let mut buf = String::new();

        gen_flag_str_serializer! {
            buf <- "CAN_CREATE_TCP"       ? self.can_create_tcp(),
            buf <- "CAN_SELECT_TCP"       ? self.can_select_tcp_port(),
            
            buf <- "CAN_CREATE_UDP"       ? self.can_create_udp(),
            buf <- "CAN_SELECT_UDP"       ? self.can_select_udp_port(),

            buf <- "CAN_CREATE_HTTP"      ? self.can_create_http(),
            buf <- "CAN_SELECT_HTTP_PATH" ? self.can_select_http_path()
        }

        if buf.is_empty() {
            return "No rights".to_owned()
        } else {
            buf
        }
    }
}

pub trait RightsToString {
    fn show_rights(self) -> String;
}

pub trait RightsFlags {
    fn can_select_udp_port(self)  -> bool;
    fn can_select_tcp_port(self)  -> bool;
    fn can_select_http_path(self) -> bool;

    fn can_create_udp(self)  -> bool;
    fn can_create_tcp(self)  -> bool;
    fn can_create_http(self) -> bool;
}

pub trait TypeFlags {
    fn is_short(self) -> bool;
    fn is_c_short(self) -> bool;
    fn is_compressed(self) -> bool;
}

pub trait ErrorToString {
    fn to_error_string(self) -> &'static str;
}
