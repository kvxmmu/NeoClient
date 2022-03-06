pub mod rights_flags {
    pub const CREATE_SERVER: u8 = 1 << 0;
    pub const SELECT_PORT: u8   = 1 << 1;

    #[inline(always)]
    pub fn all() -> u8 {
        CREATE_SERVER | SELECT_PORT
    }
}

pub struct UserMeta {
    pub rights: u8,
}

impl UserMeta {
    pub fn rights_to_string(rights: u8) -> String {
        let mut buf = String::new();

        if (rights & rights_flags::CREATE_SERVER) != 0 {
            buf.push_str(" | CREATE_SERVER");
        }
        if (rights & rights_flags::SELECT_PORT) != 0 {
            buf.push_str(" | SELECT_PORT");
        }

        String::from(&buf[3..])
    }

    #[inline]
    pub fn replace_rights(
        &mut self,
        flags: u8
    ) -> u8 {
        self.rights = flags;
        flags
    }

    #[inline]
    pub fn reject_rights(
        &mut self,
        flags: u8
    ) -> u8 {
        self.rights &= !flags;
        self.rights
    }

    #[inline]
    pub fn grant_rights(
        &mut self,
        flags: u8
    ) -> u8 {
        self.rights |= flags;
        self.rights
    }

    #[inline]
    pub fn has_rights(
        &self,
        flags: u8
    ) -> bool {
        (self.rights & flags) == flags
    }

    pub fn new(
        rights: u8
    ) -> Self {
        Self{
            rights
        }
    }
}
