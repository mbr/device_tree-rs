use alloc::vec::Vec;

pub struct StringTable {
    pub buffer: Vec<u8>,
}

impl Default for StringTable {
    fn default() -> Self {
        StringTable::new()
    }
}

impl StringTable {
    pub fn new() -> StringTable {
        StringTable { buffer: Vec::new() }
    }

    pub fn add_string(&mut self, val: &str) -> u32 {
        let offset = self.buffer.len();
        self.buffer.extend(val.bytes());
        self.buffer.push(0);
        offset as u32
    }
}
