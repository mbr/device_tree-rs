// helper function to convert to big endian
#[cfg(target_endian = "little")]
#[inline]
pub fn be_u32(raw: u32) -> u32 {
    ((raw >> 24) & 0xff
     |(raw >> 8) & 0xff00
     |(raw << 8) & 0xff0000
     |(raw << 24)  & 0xff000000)
}

#[cfg(target_endian = "big")]
#[inline]
pub fn be_u32(raw: u32) -> u32 {
    raw
}
