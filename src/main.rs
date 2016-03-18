extern crate byteorder;
extern crate core;
extern crate clap;

use byteorder::{ByteOrder, BigEndian, ReadBytesExt};
use core::{io, result};
use std::fs;

enum ParseError {
    InvalidMagic
}

type Result<T> = result::Result<T, ParseError>;

struct DeviceTreeParser<'a>
{
    f: &'a mut fs::File,
}

impl<'a> DeviceTreeParser<'a> {
    pub fn new(f: &'a mut fs::File) -> DeviceTreeParser<'a> {
        DeviceTreeParser{
            f: f,
        }
    }

    pub fn parse<E: ByteOrder>(&mut self) -> Result<()> {
        // first, read the header
        let magic = try!(self.f.read_u32::<E>());

        let totalsize = self.f.read_u32::<E>().unwrap();

        let off_dt_struct = 0;
        let off_dt_strings = 0;
        let off_mem_rsvmap = 0;
        let version = 0;
        let last_comp_version = 0;

        // version 2 fields
        let boot_cpuid_phys = 0;

        // version 3 fields
        let size_dt_strings = 0;

        // version 17 fields
        let size_dt_struct = 0;

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
