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
    InvalidTag,
    UnexpectedTag,
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
    Property,
    BeginNode,
    EndNode,
    End,
    Magic,
}

impl Tag {
    fn from_u32(val: u32) -> Result<Tag> {
        match val {
            0x01 => Ok(Tag::BeginNode),
            0x02 => Ok(Tag::EndNode),
            0x03 => Ok(Tag::Property),
            0x09 => Ok(Tag::End),
            0xd00dfeed => Ok(Tag::Magic),
            _ => {
                println!("INVALID TAG {:#X}", val);
                Err(ParseError::InvalidTag)
            },
        }
    }
}

impl<'a> DeviceTreeParser<'a> {
    pub fn new(buf: &'a [u8]) -> DeviceTreeParser<'a> {
        DeviceTreeParser{
            buf: MiniStream::new(buf),
        }
    }

    pub fn pos(&self) -> usize {
        self.buf.pos()
    }

    fn tag(&mut self) -> Result<Tag> {
        Ok(try!(Tag::from_u32(try!(self.buf.read_u32_le()))))
    }

    fn peek_tag(&self) -> Result<Tag> {
        Ok(try!(Tag::from_u32(try!(self.buf.peek_u32_le()))))
    }

    fn accept_tag(&mut self, tag: Tag) -> Result<bool> {
        let t = try!(self.peek_tag());

        if t == tag {
            self.tag();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn expect_tag(&mut self, tag: Tag) -> Result<Tag> {
        if try!(self.tag()) != tag {
            Err(ParseError::UnexpectedTag)
        } else {
            Ok(tag)
        }
    }

    fn string0(&mut self) -> Result<&[u8]> {
        let start = self.buf.pos();
        let mut num_blocks = 0;
        let mut offset;

        'search: loop {
            let block = try!(self.buf.read_bytes(4));
            num_blocks += 1;

            for i in 0..4 {
                if block[i] == 0 {
                    offset = 4-i;
                    break 'search;
                }
            }
        }

        try!(self.buf.seek(start));
        let data = try!(self.buf.read_bytes(num_blocks * 4));
        Ok(&data[..data.len()-offset])
    }

    fn structure(&mut self) -> Result<Option<()>> {
        if try!(self.accept_tag(Tag::BeginNode)) {
            let name = try!(self.string0()).to_owned();
            println!("NAME {:?}", name);

            println!("AFTER NAME POS IS {:#x}", self.pos());
            while try!(self.accept_tag(Tag::Property)) {
                let val_size = try!(self.buf.read_u32_le());
                let val_offset = try!(self.buf.read_u32_le());

                // specs unclear, now following "proeprty value data if any"
                let val_data = try!(self.string0());
                println!("FOUND PROPERTY {} {} {:?}",
                        val_size, val_offset, val_data);
            }
            println!("Done READING PROPS");
            Ok(Some(()))
        } else {
            Ok(None)
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

        println!("{:?}",header);

        // read structure first
        try!(self.buf.seek(off_dt_struct as usize));
        try!(self.structure());


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

    println!("{:?} @ {:#X}", parser.parse(), parser.pos());
}
