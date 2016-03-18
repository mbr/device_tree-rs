extern crate byteorder;
extern crate core;
extern crate clap;

use byteorder::{ByteOrder, BigEndian, ReadBytesExt};
use core::result;

enum MiniStreamReadError {
    ReadPastEnd
}

struct MiniStream<'a>{
    buf: &'a [u8],
    pos: usize,
}

impl<'a> MiniStream<'a> {
    fn new(buf: &'a [u8]) -> MiniStream<'a> {
        MiniStream{
            buf: buf,
            pos: 0,
        }
    }

    fn read_byte(&mut self) -> result::Result<u8, MiniStreamReadError> {
        if self.pos+1 < self.buf.len() {
            let byte = self.buf[self.pos];
            self.pos += 1;
            Ok(byte)
        } else {
            Err(MiniStreamReadError::ReadPastEnd)
        }
    }

    fn read_u32_le(&mut self) -> result::Result<u32, MiniStreamReadError> {
        if self.pos + 4 < self.buf.len() {
            let val: u32 = unsafe {
                *(self.buf[self.pos..(self.pos+4)].as_ptr() as *const u32)
            };
            self.pos += 4;

            // FIXME: determine endianness and properly convert
            Ok((val >> 24) & 0xff
              |(val >> 8) & 0xff00
              |(val << 8) & 0xff0000
              |(val << 24)  & 0xff000000)
        } else {
            Err(MiniStreamReadError::ReadPastEnd)
        }
    }
}

impl From<MiniStreamReadError> for ParseError {
    fn from(e: MiniStreamReadError) -> ParseError {
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
    ReadError
}

type Result<T> = result::Result<T, ParseError>;

struct DeviceTreeParser<'a>
{
    buf: MiniStream<'a>,
}

impl<'a> DeviceTreeParser<'a> {
    pub fn new(buf: &'a [u8]) -> DeviceTreeParser<'a> {
        DeviceTreeParser{
            buf: MiniStream::new(buf),
        }
    }

    pub fn parse<E: ByteOrder>(&mut self) -> Result<()> {
        // // first, read the header
        let magic = try!(self.buf.read_u32_le());

        if magic != 0xd00dfeed {
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
        println!("header {:?}", header);
        Ok(())
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
    input.read_to_end(&mut buf);

    let mut parser = DeviceTreeParser::new(&mut buf);

    println!("{:?}", parser.parse::<BigEndian>());
}
