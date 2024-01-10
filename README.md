# `device_tree`: Parse flattened Linux device trees

Device trees are used to describe a lot of hardware, especially in the ARM embedded world and are also used to boot Linux on these device. A device tree describes addresses and other attributes for many parts on these boards.

This library allows parsing the so-called flattened device trees, which are the compiled binary forms of these trees. Users can create these files using the `dtc` (device tree compiler) utility:

```bash
# If your DTS includes C pre-processor directives (e.g. #include <...>), run the `cpp` utillity
cpp -E -P -Wp,-I<include-dir> /path/to/some.dts > processed.dts

# Run the `dtc` utility to "flatten" the device tree
dtc -I dts -O [dts,dtb] -i <include-dir> -o flattened.[dts,dtb] processed.dts
```

To read more about device trees, check out [the kernel docs](https://git.kernel.org/cgit/linux/kernel/git/torvalds/linux.git/plain/Documentation/devicetree/booting-without-of.txt?id=HEAD).

Some example device trees to try out are [the Raspberry Pi ones](https://github.com/raspberrypi/firmware/tree/master/boot).

The library does not use `std`, just `core`.

# Examples

```rust
# use std::{fs, io::Read};
fn main() {
    // read file into memory
    let mut input = fs::File::open("examples/bcm2709-rpi-2-b.dtb").unwrap();
    let mut buf = Vec::new();
    input.read_to_end(&mut buf).unwrap();

    let dt = device_tree::DeviceTree::load(buf.as_slice ()).unwrap();
    println!("{:?}", dt);
}
```
