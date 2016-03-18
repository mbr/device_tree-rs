use core::str;

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

pub fn from_utf8_safe(v: &[u8]) -> &str {
    match str::from_utf8(v) {
        Ok(s) => s,
        Err(_) => "(invalid utf8)"
    }
}

#[derive(Debug)]
pub enum ReadError {
    ReadPastEnd
}

pub struct SliceReader<'a>{
    buf: &'a [u8],
    pos: usize,
}
