extern crate core;
extern crate clap;

use core::result;
pub mod util;
use util::{MiniStream, MiniStreamReadError};

impl From<MiniStreamReadError> for ParseError {
    fn from(_: MiniStreamReadError) -> ParseError {
        ParseError::ReadError
    }
}

// we only use std::fs in our commandline frontend. the parser uses libcore
// only
use std::fs;
use std::io::Read;

#[derive(Debug)]
enum ParseError {
    InvalidMagic,
    ReadError,
    InvalidTagError,
}

type Result<T> = result::Result<T, ParseError>;

struct DeviceTreeParser<'a>
{
    buf: MiniStream<'a>,
}

#[derive(Debug)]
struct DeviceTree {
    header: DeviceTreeHeader
}

#[derive(Debug, PartialEq)]
enum Tag {
    Prop,
    EndNode,
    End,
    Magic,
}

impl<'a> DeviceTreeParser<'a> {
    pub fn new(buf: &'a [u8]) -> DeviceTreeParser<'a> {
        DeviceTreeParser{
            buf: MiniStream::new(buf),
        }
    }

    fn tag(&mut self) -> Result<Tag> {
        match try!(self.buf.read_u32_le()) {
            0x02 => Ok(Tag::EndNode),
            0x03 => Ok(Tag::Prop),
            0x09 => Ok(Tag::End),
            0xd00dfeed => Ok(Tag::Magic),
            _ => Err(ParseError::InvalidTagError),
        }
    }

    pub fn parse(&mut self) -> Result<DeviceTree> {
        // // first, read the header
        if try!(self.tag()) != Tag::Magic {
            return Err(ParseError::InvalidMagic);
        }

        let totalsize = try!(self.buf.read_u32_le());
        let off_dt_struct = try!(self.buf.read_u32_le());
        let off_dt_strings = try!(self.buf.read_u32_le());
        let off_mem_rsvmap = try!(self.buf.read_u32_le());
        let version = try!(self.buf.read_u32_le());
        let last_comp_version = try!(self.buf.read_u32_le());

        let mut boot_cpuid_phys = 0;
        if version > 2 {
            boot_cpuid_phys = try!(self.buf.read_u32_le());
        }

        // version 3 fields
        let mut size_dt_strings = 0;
        if version > 3 {
            size_dt_strings = try!(self.buf.read_u32_le())
        }

        // version 17 fields
        let mut size_dt_struct = 0;
        if version > 17 {
            size_dt_struct = try!(self.buf.read_u32_le());
        }

        let header = DeviceTreeHeader{
            totalsize: totalsize,
            off_dt_struct: off_dt_struct,
            off_dt_strings: off_dt_strings,
            off_mem_rsvmap: off_mem_rsvmap,
            version: version,
            last_comp_version: last_comp_version,

            // version 2 fields
            boot_cpuid_phys: boot_cpuid_phys,

            // version 3 fields
            size_dt_strings: size_dt_strings,

            // version 17 fields
            size_dt_struct: size_dt_struct,
        };

        // read structure first
        try!(self.buf.seek(off_dt_struct as usize));
        println!("{:?}", self.buf.read_byte());
        println!("{:?}", self.buf.read_byte());
        println!("{:?}", self.buf.read_byte());
        println!("{:?}", self.buf.read_byte());

        Ok(DeviceTree{
            header: header
        })
    }
}

#[derive(Debug)]
struct DeviceTreeHeader {
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



fn main() {
    let matches = clap::App::new("device-tree-parser")
                                .arg(clap::Arg::with_name("input_file")
                                    .help("Flattened device tree (.dtb)")
                                    .takes_value(true)
                                    .required(true)
                                    .value_name("FILE"))
                                .get_matches();

    // read file into memory
    let mut input = fs::File::open(matches.value_of("input_file").unwrap())
                                  .unwrap();
    let mut buf = Vec::new();
    input.read_to_end(&mut buf).unwrap();

    let mut parser = DeviceTreeParser::new(&mut buf);

    println!("{:?}", parser.parse());
}
