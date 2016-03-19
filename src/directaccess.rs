use core::mem::size_of;
use core::{fmt, iter, result, str};

use util::{align, be_u32, SliceRead, SliceReadError};

const MAGIC_NUMBER     : u32 = 0xd00dfeed;
const SUPPORTED_VERSION: u32 = 17;
const OF_DT_BEGIN_NODE : u32 = 0x00000001;
const OF_DT_END_NODE   : u32 = 0x00000002;
const OF_DT_PROP       : u32 = 0x00000003;

#[derive(Debug)]
pub enum DeviceTreeError {
    CantFitHeader,
    NoMagicNumberFound,
    SizeMismatch,
    UnsupportedVersion,
    RunawayString,
    Utf8Error,
    SliceReadError,
    InvalidTag,
}

pub type Result<T> = result::Result<T, DeviceTreeError>;

pub struct DeviceTree<'a> {
    buffer: &'a [u8]
}

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

pub struct Node<'a> {
    buffer: &'a [u8],
    start: usize,
    name_end: usize,
}

struct PropertyIter<'a> {
    buffer: &'a [u8],
    pos: usize,
}

struct Property<'a> {
    buffer: &'a [u8],
    start: usize,
}

impl From<str::Utf8Error> for DeviceTreeError {
    fn from(_: str::Utf8Error) -> DeviceTreeError {
        DeviceTreeError::Utf8Error
    }
}

impl From<SliceReadError> for DeviceTreeError {
    fn from(_: SliceReadError) -> DeviceTreeError {
        DeviceTreeError::SliceReadError
    }
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

            if header.version() != SUPPORTED_VERSION &&
               header.last_comp_version() != SUPPORTED_VERSION {
                return Err(DeviceTreeError::UnsupportedVersion)
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

    pub fn root(&self) -> Result<Node> {
        Node::new(self.buffer, self.header().off_dt_struct())
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

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Header {{ magic_number: {:#x}, total_size: {}, \
                            off_dt_struct: {}, off_dt_strings: {}, \
                            off_mem_rsvmap: {}, version: {}, \
                            last_comp_version: {}, boot_cpuid_phys: {}, \
                            size_dt_strings: {}, size_dt_struct: {}, \
                    }}", self.magic_number(), self.total_size(),
                         self.off_dt_struct(), self.off_dt_strings(),
                         self.off_mem_rsvmap(), self.version(),
                         self.last_comp_version(), self.boot_cpuid_phys(),
                         self.size_dt_strings(), self.size_dt_struct())
    }
}


impl<'a> PropertyIter<'a> {
    fn new(buffer: &'a [u8], pos: usize) -> PropertyIter<'a> {
        PropertyIter{
            buffer: buffer,
            pos: pos,
        }
    }
}

impl<'a> iter::Iterator for PropertyIter<'a> {
    type Item = Result<Property<'a>>;

    fn next(&mut self) -> Option<Result<Property<'a>>> {
        // look for opening tag
        if trysome!(self.buffer.read_be_u32(self.pos)) != OF_DT_PROP {
            return None  // no opening tag, so no property
        }

        let val_size = trysome!(self.buffer.read_be_u32(self.pos + 4)) as usize;
        // ignore the name offset, Property will read it iself

        // at pos+12, the value starts
        let prop_end = self.pos + 12 + val_size;
        if ! prop_end < self.buffer.len() {
            return Some(Err(DeviceTreeError::SizeMismatch));
        }

        let prop = Property{
            buffer: self.buffer,
            start: self.pos
        };

        self.pos = align(prop_end, 4);

        Some(Ok(prop))
    }
}

impl<'a> Node<'a> {
    pub fn new(buffer: &'a [u8], start: usize) -> Result<Node<'a>> {
        if try!(buffer.read_be_u32(start)) != OF_DT_BEGIN_NODE {
            return Err(DeviceTreeError::InvalidTag)
        }

        let name = try!(buffer.read_bstring0(start+4));
        let name_end = start + 4 + name.len();

        // after 0 byte, align to 4-byte boundary
        let prop_start = align(name_end + 1, 4);

        Ok(Node{
            buffer: buffer,
            start: start,
            name_end: name_end,
        })
    }

    pub fn name(&self) -> Result<&'a str> {
        Ok(try!(str::from_utf8(self.name_bytes())))
    }

    pub fn name_bytes(&self) -> &'a [u8] {
        let begin = self.start + 4;
        &self.buffer[begin..self.name_end]
    }
}

impl<'a> fmt::Debug for Node<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut name = self.name().unwrap_or("INVALID NAME");
        if name == "" {
            // root node has no name
            name = "/"
        };

        write!(f, "{}", name)
    }
}
