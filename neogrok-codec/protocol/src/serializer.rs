#[inline(always)]
pub fn pack_type(
    pkt_type: u8,
    flags: u8
) -> u8 {
    (pkt_type << 3) | flags
}
