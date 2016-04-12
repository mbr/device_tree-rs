//! Parse flattened linux device trees
//!
//! Device trees are used to describe a lot of hardware, especially in the ARM
//! embedded world and are also used to boot Linux on these device. A device
//! tree describes addresses and other attributes for many parts on these
//! boards
//!
//! This library allows parsing the so-called flattened device trees, which
//! are the compiled binary forms of these trees.
//!
//! To read more about device trees, check out
//! [the kernel docs](https://git.kernel.org/cgit/linux/kernel/git/torvalds/linux.git/plain/Documentation/devicetree/booting-without-of.txt?id=HEAD).
//! Some example device trees
//! to try out are [the Raspberry Pi ones]
//! (https://github.com/raspberrypi/firmware/tree/master/boot).
//!
//! The library does not use `std`, just `core`.
//!
//! # Examples
//!
//! ```ignore
//! fn main() {
//!     // read file into memory
//!     let mut input = fs::File::open("sample.dtb").unwrap();
//!     let mut buf = Vec::new();
//!     input.read_to_end(&mut buf).unwrap();
//!
//!     let dt = device_tree::DeviceTree::load(buf.as_slice ()).unwrap();
//!     println!("{:?}", dt);
//! }
//! ```

extern crate core;

pub mod util;

use core::str;
use util::{align, SliceRead, SliceReadError};

const MAGIC_NUMBER     : u32 = 0xd00dfeed;
const SUPPORTED_VERSION: u32 = 17;
const OF_DT_BEGIN_NODE : u32 = 0x00000001;
const OF_DT_END_NODE   : u32 = 0x00000002;
const OF_DT_PROP       : u32 = 0x00000003;


/// An error describe parsing problems when creating device trees.
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

    /// While trying to convert a string that was supposed to be ASCII, invalid
    /// utf8 sequences were encounted
    Utf8Error,

    /// The device tree version is not supported by this library.
    VersionNotSupported,
}

/// Device tree structure.
#[derive(Debug)]
pub struct DeviceTree {
    /// Version, as indicated by version header
    pub version: u32,

    /// The number of the CPU the system boots from
    pub boot_cpuid_phys: u32,

    /// A list of tuples of `(offset, length)`, indicating reserved memory
    // regions.
    pub reserved: Vec<(u64, u64)>,

    /// The root node.
    pub root: Node,
}

/// A single node in the device tree.
#[derive(Debug)]
pub struct Node {
    /// The name of the node, as it appears in the node path.
    pub name: String,

    /// A list of node properties, `(key, value)`.
    pub props: Vec<(String, Vec<u8>)>,

    /// Child nodes of this node.
    pub children: Vec<Node>,
}

#[derive(Debug)]
pub enum PropError {
    NotFound,
    Utf8Error,
    Missing0,
    SliceReadError(SliceReadError),
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
    //! Load a device tree from a memory buffer.
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

    pub fn find<'a>(&'a self, path: &str) -> Option<&'a Node> {
        // we only find root nodes on the device tree
        if ! path.starts_with('/') {
            return None
        }

        self.root.find(&path[1..])
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

    pub fn find<'a>(&'a self, path: &str) -> Option<&'a Node> {
        if path == "" {
            return Some(self)
        }

        match path.find('/') {
            Some(idx) => {
                // find should return the proper index, so we're safe to
                // use indexing here
                let (l, r) = path.split_at(idx);

                // we know that the first char of slashed is a '/'
                let subpath = &r[1..];

                for child in self.children.iter() {
                    if child.name == l {
                        return child.find(subpath);
                    }
                }

                // no matching child found
                None
            },
            None => self.children.iter().find(|n| n.name == path)
        }
    }

    pub fn has_prop(&self, name: &str) -> bool {
        if let Some(_) = self.prop_raw(name) {
            true
        } else {
            false
        }
    }

    pub fn prop_str<'a>(&'a self, name: &str) -> Result<&'a str, PropError> {
        let raw = try!(self.prop_raw(name).ok_or(PropError::NotFound));

        let l = raw.len();
        if l < 1 || raw[l-1] != 0 {
            return Err(PropError::Missing0)
        }

        Ok(try!(str::from_utf8(&raw[..(l-1)])))
    }

    pub fn prop_raw<'a>(&'a self, name: &str) -> Option<&'a Vec<u8>> {
        for &(ref key, ref val) in self.props.iter() {
            if key == name {
                return Some(val)
            }
        }
        None
    }

    pub fn prop_u64(&self, name: &str) -> Result<u64, PropError> {
        let raw = try!(self.prop_raw(name).ok_or(PropError::NotFound));

        Ok(try!(raw.as_slice().read_be_u64(0)))
    }

    pub fn prop_u32(&self, name: &str) -> Result<u32, PropError> {
        let raw = try!(self.prop_raw(name).ok_or(PropError::NotFound));

        Ok(try!(raw.as_slice().read_be_u32(0)))
    }
}

impl From<str::Utf8Error> for PropError {
    fn from(_: str::Utf8Error) -> PropError {
        PropError::Utf8Error
    }
}

impl From<SliceReadError> for PropError {
    fn from(e: SliceReadError) -> PropError {
        PropError::SliceReadError(e)
    }
}
