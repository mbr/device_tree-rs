extern crate core;

mod util;

use core::str;
use util::{align, SliceRead, SliceReadError};

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

    /// While trying to convert a string that was supposed to be ascii, invalid
    /// utf8 sequences were encounted
    Utf8Error,

    /// The device tree version is not supported by this library.
    VersionNotSupported,
}

#[derive(Debug)]
pub struct DeviceTree {
    version: u32,
    boot_cpuid_phys: u32,
    reserved: Vec<(u64, u64)>,
    root: Node,
}


#[derive(Debug)]
pub struct Node {
    name: String,
    props: Vec<(String, Vec<u8>)>,
    children: Vec<Node>,
}


impl From<SliceReadError> for DeviceTreeError {
    fn from(e: SliceReadError) -> DeviceTreeError {
        DeviceTreeError::SliceReadError(e)
    }
}

impl From<str::Utf8Error> for DeviceTreeError {
    fn from(_: str::Utf8Error) -> DeviceTreeError {
        DeviceTreeError::Utf8Error
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
        let off_mem_rsvmap = try!(buffer.read_be_u32(16)) as usize;
        let boot_cpuid_phys = try!(buffer.read_be_u32(28));

        // load reserved memory list
        let mut reserved = Vec::new();
        let mut pos = off_mem_rsvmap;

        loop {
            let offset = try!(buffer.read_be_u64(pos));
            pos += 8;
            let size = try!(buffer.read_be_u64(pos));
            pos += 8;

            reserved.push((offset, size));

            if size == 0 {
                break;
            }
        }

        let (_, root) = try!(Node::load(buffer, off_dt_struct, off_dt_strings));

        Ok(DeviceTree{
            version: version,
            boot_cpuid_phys: boot_cpuid_phys,
            reserved: reserved,
            root: root,
        })
    }
}


impl Node {
    fn load(buffer: &[u8], start: usize, off_dt_strings: usize)
    -> Result<(usize, Node), DeviceTreeError> {
        // check for DT_BEGIN_NODE
        if try!(buffer.read_be_u32(start)) != OF_DT_BEGIN_NODE {
            return Err(DeviceTreeError::ParseError(start))
        }

        let raw_name = try!(buffer.read_bstring0(start+4));

        // read all the props
        let mut pos = align(start + 4 + raw_name.len() + 1, 4);

        let mut props = Vec::new();

        while try!(buffer.read_be_u32(pos)) == OF_DT_PROP {
            let val_size = try!(buffer.read_be_u32(pos+4)) as usize;
            let name_offset = try!(buffer.read_be_u32(pos+8)) as usize;

            // get value slice
            let val_start = pos + 12;
            let val_end = val_start + val_size;
            let val = try!(buffer.subslice(val_start, val_end));

            // lookup name in strings table
            let prop_name = try!(
                buffer.read_bstring0(off_dt_strings + name_offset)
            );

            props.push((
                try!(str::from_utf8(prop_name)).to_owned(),
                val.to_owned(),
            ));

            pos = align(val_end, 4);
        }

        // finally, parse children
        let mut children = Vec::new();

        while try!(buffer.read_be_u32(pos)) == OF_DT_BEGIN_NODE {
            let (new_pos, child_node) = try!(Node::load(buffer, pos,
                off_dt_strings));
            pos = new_pos;

            children.push(child_node);
        }

        if try!(buffer.read_be_u32(pos)) != OF_DT_END_NODE {
            return Err(DeviceTreeError::ParseError(pos))
        }

        pos += 4;

        Ok((pos, Node{
            name: try!(str::from_utf8(raw_name)).to_owned(),
            props: props,
            children: children,
        }))
    }
}
