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
    CantFitHeader,
    NoMagicNumberFound,
    SizeMismatch,
}

pub type Result<T> = result::Result<T, DeviceTreeError>;

pub struct DeviceTree<'a> {
    buffer: &'a [u8]
}

#[derive(Debug)]
pub struct Header {
    magic_number: u32,

    totalsize: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    off_mem_rsvmap: u32,
    version: u32,
    last_comp_version: u32,

    // version 2 fields
    boot_cpuid_phys: u32,

    // version 3 fields
    size_dt_strings: u32,

    // version 17 fields
    size_dt_struct: u32,
}

impl<'a> DeviceTree<'a> {
    pub fn new(buffer: &'a [u8]) -> Result<DeviceTree<'a>> {
        if buffer.len() < size_of::<Header>() {
            return Err(DeviceTreeError::CantFitHeader)
        };

        let dt = DeviceTree{
            buffer: buffer
        };

        {
            let header = dt.header();

            // check magic numbers is present
            if header.magic_number() != MAGIC_NUMBER {
                return Err(DeviceTreeError::NoMagicNumberFound);
            }

            // ensure sizes check out
            if header.total_size() != buffer.len() {
                return Err(DeviceTreeError::SizeMismatch);
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

    fn total_size(&self) -> usize {
        be_u32(self.totalsize) as usize
    }

    fn off_dt_struct(&self) -> usize {
        be_u32(self.off_dt_struct) as usize
    }

    fn off_dt_strings(&self) -> usize {
        be_u32(self.off_dt_strings) as usize
    }

    fn off_mem_rsvmap(&self) -> usize {
        be_u32(self.off_mem_rsvmap) as usize
    }

    pub fn version(&self) -> u32 {
        be_u32(self.version)
    }

    pub fn last_comp_version(&self) -> u32 {
        be_u32(self.last_comp_version)
    }

    pub fn boot_cpuid_phys(&self) -> u32 {
        be_u32(self.boot_cpuid_phys)
    }

    fn size_dt_strings(&self) -> u32 {
        be_u32(self.size_dt_strings)
    }

    fn size_dt_struct(&self) -> u32 {
        be_u32(self.size_dt_struct)
    }
}
