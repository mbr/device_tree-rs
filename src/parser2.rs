use util::{SliceRead, SliceReadError};

const MAGIC_NUMBER     : u32 = 0xd00dfeed;
const SUPPORTED_VERSION: u32 = 17;
const OF_DT_BEGIN_NODE : u32 = 0x00000001;
const OF_DT_END_NODE   : u32 = 0x00000002;
const OF_DT_PROP       : u32 = 0x00000003;


#[derive(Debug)]
pub enum DeviceTreeError {
    /// The magic number `MAGIC_NUMBER` was not found at the start of the
    /// structure.
    InvalidMagicNumber,

    /// An offset or size found inside the device tree is outside of what was
    /// supplied to `load()`.
    SizeMismatch,

    /// Failed to read data from slice.
    SliceReadError(SliceReadError),

    /// The data format was not as expected at the given position
    ParseError(usize),

    /// The device tree version is not supported by this library.
    VersionNotSupported,
}

#[derive(Debug)]
pub struct DeviceTree {
    version: u32,
    boot_cpuid_phys: u32,
    root: Node,
}


#[derive(Debug)]
pub struct Node {
    name: Vec<u8>,
}


impl From<SliceReadError> for DeviceTreeError {
    fn from(e: SliceReadError) -> DeviceTreeError {
        DeviceTreeError::SliceReadError(e)
    }
}


impl DeviceTree {
    pub fn load(buffer: &[u8]) -> Result<DeviceTree, DeviceTreeError> {
        //  0  magic_number: u32,

        //  4  totalsize: u32,
        //  8  off_dt_struct: u32,
        // 12  off_dt_strings: u32,
        // 16  off_mem_rsvmap: u32,
        // 20  version: u32,
        // 24  last_comp_version: u32,

        // // version 2 fields
        // 28  boot_cpuid_phys: u32,

        // // version 3 fields
        // 32  size_dt_strings: u32,

        // // version 17 fields
        // 36  size_dt_struct: u32,

        if try!(buffer.read_be_u32(0)) != MAGIC_NUMBER {
            return Err(DeviceTreeError::InvalidMagicNumber)
        }

        // check total size
        if try!(buffer.read_be_u32(4)) as usize != buffer.len() {
            return Err(DeviceTreeError::SizeMismatch);
        }

        // check version
        let version = try!(buffer.read_be_u32(20));
        if version != SUPPORTED_VERSION {
            return Err(DeviceTreeError::VersionNotSupported);
        }

        let off_dt_struct = try!(buffer.read_be_u32(8)) as usize;
        let off_dt_strings = try!(buffer.read_be_u32(12)) as usize;
        let boot_cpuid_phys = try!(buffer.read_be_u32(28));

        let root = try!(Node::load(buffer, off_dt_struct));

        Ok(DeviceTree{
            version: version,
            boot_cpuid_phys: boot_cpuid_phys,
            root: root,
        })
    }
}


impl Node {
    fn load(buffer: &[u8], start: usize) -> Result<Node, DeviceTreeError> {
        // check for DT_BEGIN_NODE
        if try!(buffer.read_be_u32(start)) != OF_DT_BEGIN_NODE {
            return Err(DeviceTreeError::ParseError(start))
        }

        let name = try!(buffer.read_bstring0(start+4)).to_owned();

        Ok(Node{
            name: name
        })
    }
}
