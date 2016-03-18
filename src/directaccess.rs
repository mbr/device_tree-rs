use core::mem::size_of;
use core::result;

const MAGIC_NUMBER: u32 = 0xd00dfeed;

// helper function to convert to big endian
#[cfg(target_endian = "little")]
#[inline]
fn be_u32(raw: u32) -> u32 {
    ((raw >> 24) & 0xff
     |(raw >> 8) & 0xff00
     |(raw << 8) & 0xff0000
     |(raw << 24)  & 0xff000000)
}

#[cfg(target_endian = "big")]
#[inline]
fn be_u32(raw: u32) -> u32 {
    raw
}

#[derive(Debug)]
pub enum DeviceTreeError {
    BufferTooSmall,
    NoMagicNumberFound,
}

pub type Result<T> = result::Result<T, DeviceTreeError>;

pub struct DeviceTree<'a> {
    buffer: &'a [u8]
}

#[derive(Debug)]
pub struct Header {
    magic_number: u32,
}

impl<'a> DeviceTree<'a> {
    pub fn new(buffer: &'a [u8]) -> Result<DeviceTree<'a>> {
        if buffer.len() < size_of::<Header>() {
            return Err(DeviceTreeError::BufferTooSmall)
        };

        let dt = DeviceTree{
            buffer: buffer
        };

        {
            let header = dt.header();

            if header.magic_number() != MAGIC_NUMBER {
                return Err(DeviceTreeError::NoMagicNumberFound);
            }
        }

        Ok(dt)
    }

    pub fn header(&self) -> &Header {
        // we've checked that the buffer is large enough inside the constructor
        unsafe {
            &*(self.buffer.as_ptr() as *const Header)
        }
    }
}

impl Header {
    fn magic_number(&self) -> u32 {
        be_u32(self.magic_number)
    }
}
