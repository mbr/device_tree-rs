# `psi_device_tree`

[Device trees](https://devicetree.org) are used to describe a lot of hardware, especially in the embedded world and are used in U-Boot, Linux and other boot loaders and kernels. A device tree enumerates addresses and other attributes for peripherals, hardware decoders, processing cores and external components attached to systems on chips (SoCs) on printed circuit boards (PCBs).

This library allows parsing the so-called flattened device trees (FDTs), which are the compiled binary forms of the corresponding device tree source (DTS) files that are commonly found in the respective project, e.g., Linux. Decice tree sources are often modular, bring preprocessed and then compiled to DTBs. Users can create these files using the `dtc` (device tree compiler) utility from the U-Boot project:

```bash
# If your DTS includes C pre-processor directives (e.g. #include <...>), run the `cpp` utillity
cpp -E -P -Wp,-I<include-dir> /path/to/some.dts > processed.dts

# Run the `dtc` utility to "flatten" the device tree
dtc -I dts -O [dts,dtb] -i <include-dir> -o flattened.[dts,dtb] processed.dts
```

To read more about device trees in Linux, check out [the kernel docs](https://git.kernel.org/cgit/linux/kernel/git/torvalds/linux.git/plain/Documentation/devicetree/booting-without-of.txt?id=HEAD).

Some example device trees to try out are [the Raspberry Pi ones](https://github.com/raspberrypi/firmware/tree/master/boot).

This library does not use `std`, just `core`.

## Example usage

```rust
use std::{fs, io::Read};
use psi_device_tree::DeviceTree as DT;

fn main() {
    // read file into memory
    let mut input = fs::File::open("examples/bcm2709-rpi-2-b.dtb").unwrap();
    let mut buf = Vec::new();
    input.read_to_end(&mut buf).unwrap();

    let dt = DT::load(buf.as_slice ()).unwrap();
    println!("{dt:?}");
}
```

## CLI Tools

### `extract-dtb`

extract .dtb files from images

```
$> cargo run extract-dtb
```
