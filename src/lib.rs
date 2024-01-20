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
//! ```rust
//! # use std::{fs, io::Read};
//! use psi_device_tree::DeviceTree;
//!
//! fn main() {
//!     // read file into memory
//!     let mut input = fs::File::open("examples/bcm2709-rpi-2-b.dtb").unwrap();
//!     let mut buf = Vec::new();
//!     input.read_to_end(&mut buf).unwrap();
//!
//!     let dt = DeviceTree::load(buf.as_slice ()).unwrap();
//!     println!("{:?}", dt);
//! }
//! ```

#![no_std]

extern crate alloc;
extern crate hashbrown;

mod error;
pub mod util;

use alloc::{borrow::ToOwned, string::String, vec::Vec};
use core::str;
use serde::{Deserialize, Serialize};

pub use error::*;
use util::{align, SliceRead, VecWrite};

#[cfg(not(feature = "string-dedup"))]
mod string_table;
#[cfg(feature = "string-dedup")]
mod advanced_string_table;

#[cfg(not(feature = "string-dedup"))]
use string_table::StringTable;

#[cfg(feature = "string-dedup")]
use advanced_string_table::StringTable;

const MAGIC_NUMBER: u32 = 0xd00dfeed;
const SUPPORTED_VERSION: u32 = 17;
const COMPAT_VERSION: u32 = 16;
const OF_DT_BEGIN_NODE: u32 = 0x00000001;
const OF_DT_END_NODE: u32 = 0x00000002;
const OF_DT_PROP: u32 = 0x00000003;
const OF_DT_END: u32 = 0x00000009;

/// Device tree structure.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// The name of the node, as it appears in the node path.
    pub name: String,

    /// A list of node properties, `(key, value)`.
    pub props: Vec<(String, Vec<u8>)>,

    /// Child nodes of this node.
    pub children: Vec<Node>,
}

impl DeviceTree {
    //! Load a device tree from a memory buffer.
    pub fn load(buffer: &[u8]) -> Result<DeviceTree> {
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

        if buffer.read_be_u32(0)? != MAGIC_NUMBER {
            return Err(Error::InvalidMagicNumber);
        }

        // check total size
        if buffer.read_be_u32(4)? as usize != buffer.len() {
            return Err(Error::SizeMismatch);
        }

        // check version
        let version = buffer.read_be_u32(20)?;
        if version != SUPPORTED_VERSION {
            return Err(Error::VersionNotSupported);
        }

        let off_dt_struct = buffer.read_be_u32(8)? as usize;
        let off_dt_strings = buffer.read_be_u32(12)? as usize;
        let off_mem_rsvmap = buffer.read_be_u32(16)? as usize;
        let boot_cpuid_phys = buffer.read_be_u32(28)?;

        // load reserved memory list
        let mut reserved = Vec::new();
        let mut pos = off_mem_rsvmap;

        loop {
            let offset = buffer.read_be_u64(pos)?;
            pos += 8;
            let size = buffer.read_be_u64(pos)?;
            pos += 8;

            reserved.push((offset, size));

            if size == 0 {
                break;
            }
        }

        let (_, root) = Node::load(buffer, off_dt_struct, off_dt_strings)?;

        Ok(DeviceTree {
            version,
            boot_cpuid_phys,
            reserved,
            root,
        })
    }

    pub fn find<'a>(&'a self, path: &str) -> Option<&'a Node> {
        // we only find root nodes on the device tree
        if !path.starts_with('/') {
            return None;
        }

        self.root.find(&path[1..])
    }

    pub fn store(&self) -> Result<Vec<u8>> {
        let mut dtb = Vec::new();
        let mut strings = StringTable::new();

        // Magic
        let len = dtb.len();
        dtb.write_be_u32(len, MAGIC_NUMBER)?;

        let size_off = dtb.len();
        dtb.write_be_u32(size_off, 0)?; // Fill in size later
        let off_dt_struct = dtb.len();
        dtb.write_be_u32(off_dt_struct, 0)?; // Fill in off_dt_struct later
        let off_dt_strings = dtb.len();
        dtb.write_be_u32(off_dt_strings, 0)?; // Fill in off_dt_strings later
        let off_mem_rsvmap = dtb.len();
        dtb.write_be_u32(off_mem_rsvmap, 0)?; // Fill in off_mem_rsvmap later

        // Version
        let len = dtb.len();
        dtb.write_be_u32(len, SUPPORTED_VERSION)?;
        // Last comp version
        let len = dtb.len();
        dtb.write_be_u32(len, COMPAT_VERSION)?;
        // boot_cpuid_phys
        let len = dtb.len();
        dtb.write_be_u32(len, self.boot_cpuid_phys)?;

        let off_size_strings = dtb.len();
        dtb.write_be_u32(off_size_strings, 0)?; // Fill in size_dt_strings later
        let off_size_struct = dtb.len();
        dtb.write_be_u32(off_size_struct, 0)?; // Fill in size_dt_struct later

        // Memory Reservation Block
        dtb.pad(8)?;
        let len = dtb.len();
        dtb.write_be_u32(off_mem_rsvmap, len as u32)?;
        for reservation in self.reserved.iter() {
            // address
            let len = dtb.len();
            dtb.write_be_u64(len, reservation.0)?;
            // size
            let len = dtb.len();
            dtb.write_be_u64(len, reservation.1)?;
        }

        // Structure Block
        dtb.pad(4)?;
        let structure_start = dtb.len();
        dtb.write_be_u32(off_dt_struct, structure_start as u32)?;
        self.root.store(&mut dtb, &mut strings)?;

        dtb.pad(4)?;
        let len = dtb.len();
        dtb.write_be_u32(len, OF_DT_END)?;

        let len = dtb.len();
        dtb.write_be_u32(off_size_struct, (len - structure_start) as u32)?;
        dtb.write_be_u32(off_size_strings, strings.buffer.len() as u32)?;

        // Strings Block
        dtb.pad(4)?;
        let len = dtb.len();
        dtb.write_be_u32(off_dt_strings, len as u32)?;
        dtb.extend_from_slice(&strings.buffer);

        let len = dtb.len();
        dtb.write_be_u32(size_off, len as u32)?;

        Ok(dtb)
    }
}

impl Node {
    fn load(
        buffer: &[u8],
        start: usize,
        off_dt_strings: usize,
    ) -> Result<(usize, Node)> {
        // check for DT_BEGIN_NODE
        if buffer.read_be_u32(start)? != OF_DT_BEGIN_NODE {
            return Err(Error::ParseError(start));
        }

        let raw_name = buffer.read_bstring0(start + 4)?;

        // read all the props
        let mut pos = align(start + 4 + raw_name.len() + 1, 4);

        let mut props = Vec::new();

        while buffer.read_be_u32(pos)? == OF_DT_PROP {
            let val_size = buffer.read_be_u32(pos + 4)? as usize;
            let name_offset = buffer.read_be_u32(pos + 8)? as usize;

            // get value slice
            let val_start = pos + 12;
            let val_end = val_start + val_size;
            let val = buffer.subslice(val_start, val_end)?;

            // lookup name in strings table
            let prop_name = buffer.read_bstring0(off_dt_strings + name_offset)?;

            props.push((str::from_utf8(prop_name)?.to_owned(), val.to_owned()));

            pos = align(val_end, 4);
        }

        // finally, parse children
        let mut children = Vec::new();

        while buffer.read_be_u32(pos)? == OF_DT_BEGIN_NODE {
            let (new_pos, child_node) = Node::load(buffer, pos, off_dt_strings)?;
            pos = new_pos;

            children.push(child_node);
        }

        if buffer.read_be_u32(pos)? != OF_DT_END_NODE {
            return Err(Error::ParseError(pos));
        }

        pos += 4;

        Ok((
            pos,
            Node {
                name: str::from_utf8(raw_name)?.to_owned(),
                props,
                children,
            },
        ))
    }

    pub fn find<'a>(&'a self, path: &str) -> Option<&'a Node> {
        if path.is_empty() {
            return Some(self);
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
            }
            None => self.children.iter().find(|n| n.name == path),
        }
    }

    pub fn has_prop(&self, name: &str) -> bool {
        self.prop_raw(name).is_some()
    }

    pub fn prop_str<'a>(&'a self, name: &str) -> Result<&'a str> {
        let raw = self.prop_raw(name).ok_or(PropError::NotFound)?;

        let l = raw.len();
        if l < 1 || raw[l - 1] != 0 {
            return Err(PropError::Missing0.into());
        }

        Ok(str::from_utf8(&raw[..(l - 1)])?)
    }

    pub fn prop_raw<'a>(&'a self, name: &str) -> Option<&'a Vec<u8>> {
        for (key, val) in self.props.iter() {
            if key == name {
                return Some(val);
            }
        }
        None
    }

    pub fn prop_u64(&self, name: &str) -> Result<u64> {
        let raw = self.prop_raw(name).ok_or(PropError::NotFound)?;

        Ok(raw.as_slice().read_be_u64(0)?)
    }

    pub fn prop_u32(&self, name: &str) -> Result<u32> {
        let raw = self.prop_raw(name).ok_or(PropError::NotFound)?;

        Ok(raw.as_slice().read_be_u32(0)?)
    }

    pub fn store(
        &self,
        structure: &mut Vec<u8>,
        strings: &mut StringTable,
    ) -> Result<()> {
        structure.pad(4)?;
        let len = structure.len();
        structure.write_be_u32(len, OF_DT_BEGIN_NODE)?;

        structure.write_bstring0(&self.name)?;
        for prop in self.props.iter() {
            structure.pad(4)?;
            let len = structure.len();
            structure.write_be_u32(len, OF_DT_PROP)?;

            // Write property value length
            structure.pad(4)?;
            let len = structure.len();
            structure.write_be_u32(len, prop.1.len() as u32)?;

            // Write name offset
            structure.pad(4)?;
            let len = structure.len();
            structure.write_be_u32(len, strings.add_string(&prop.0))?;

            // Store the property value
            structure.extend_from_slice(&prop.1);
        }

        // Recurse on children
        for child in self.children.iter() {
            child.store(structure, strings)?;
        }

        structure.pad(4)?;
        let len = structure.len();
        structure.write_be_u32(len, OF_DT_END_NODE)?;
        Ok(())
    }
}
