/// Blue Protocol protocol constants and types
pub const SERVICE_UUID: u64 = 0x63335342;

pub mod packet {
    pub const COMPRESSION_FLAG: u16 = 0x8000;
    pub const TYPE_MASK: u16 = 0x7FFF;

    #[inline]
    pub fn extract_type(packet_type: u16) -> u16 {
        packet_type & TYPE_MASK
    }
}

pub mod packet_layout {
    pub const SERVER_SIGNATURE_OFFSET: usize = 5;
}

pub mod entity {
    pub const TYPE_MASK: u16 = 0xFFFF;

    #[inline]
    pub fn get_player_uid(uuid: i64) -> i64 {
        uuid >> 16
    }
}

pub mod server_detection {
    pub const SERVER_SIGNATURE: &[u8] = &[0x00, 0x63, 0x33, 0x53, 0x42, 0x00];
    pub const LOGIN_RETURN_SIGNATURE_1: &[u8] =
        &[0x00, 0x00, 0x00, 0x62, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01];
    pub const LOGIN_RETURN_SIGNATURE_2: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x0a, 0x4e];
    pub const LOGIN_RETURN_SIGNATURE_SIZE: usize = 0x62;
}

pub mod attr_type {
    pub const ATTR_NAME: i32 = 0x01;
    pub const ATTR_ID: i32 = 0x0A;
    pub const ATTR_HP: i32 = 0x2C2E;
    pub const ATTR_MAX_HP: i32 = 0x2C38;
    pub const ATTR_PROFESSION_ID: i32 = 0xDC;
    pub const ATTR_FIGHT_POINT: i32 = 0x272E;
    pub const ATTR_POS: i32 = 0x34;
}

pub mod damage {
    pub const CRIT_BIT: i32 = 0b00000001;
}
