use alloc::vec::Vec;
use hashbrown::HashMap;

pub struct StringTable {
    pub buffer: Vec<u8>,
    index: HashMap<String, u32>,
}

impl StringTable {
    pub fn new() -> StringTable {
        StringTable {
            buffer: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn add_string(&mut self, val: &str) -> u32 {
        if let Some(offset) = self.index.get(val) {
            return *offset;
        }
        let offset = self.buffer.len() as u32;
        self.buffer.extend(val.bytes());
        self.buffer.push(0);
        self.index.insert(val.to_string(), offset);
        offset
    }
}
