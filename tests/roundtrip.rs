extern crate device_tree;

use std::fs;
use std::io::{Read, Write};

use device_tree::*;

#[test]
fn roundtrip() {
    // read file into memory
    let buf = include_bytes!("../examples/bcm2709-rpi-2-b.dtb");
    let original_fdt = DeviceTree::load(buf).unwrap();

    let dtb = original_fdt.store().unwrap();
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("output.dtb")
        .unwrap();
    output.write_all(&dtb).unwrap();

    let mut input = fs::File::open("output.dtb").unwrap();
    let mut buf = Vec::new();
    input.read_to_end(&mut buf).unwrap();
    let generated_fdt = DeviceTree::load(buf.as_slice()).unwrap();

    assert!(original_fdt == generated_fdt);
}
