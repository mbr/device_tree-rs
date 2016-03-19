pub use core::{convert, fmt, option, result, str};

// helper function to convert to big endian
#[cfg(target_endian = "little")]
#[inline]
pub fn be_u32(raw: u32) -> u32 {
    ((raw >> 24) & 0xff
     |(raw >> 8) & 0xff00
     |(raw << 8) & 0xff0000
     |(raw << 24)  & 0xff000000)
}

#[cfg(target_endian = "big")]
#[inline]
pub fn be_u32(raw: u32) -> u32 {
    raw
}

#[inline]
pub fn align(val: usize, to: usize) -> usize {
    val + (to - (val % to)) % to
}

pub enum SliceReadError {
    UnexpectedEndOfInput,
}

pub type SliceReadResult<T> = Result<T, SliceReadError>;

pub trait SliceRead {
    fn read_be_u32(&self, pos: usize) -> SliceReadResult<u32>;
    fn read_bstring0(&self, pos: usize) -> SliceReadResult<&[u8]>;
}

impl<'a> SliceRead for &'a [u8] {
    fn read_be_u32(&self, pos: usize) -> SliceReadResult<u32> {
        // check size is valid
        if ! pos+4 < self.len() {
            return Err(SliceReadError::UnexpectedEndOfInput)
        }

        Ok(
            (self[pos] as u32) << 24
            | (self[pos+1] as u32) << 16
            | (self[pos+2] as u32) << 8
            | (self[pos+3] as u32)
        )
    }

    fn read_bstring0(&self, pos: usize) -> SliceReadResult<&[u8]> {
        let mut cur = pos;
        while cur < self.len() {
            if self[cur] == 0 {
                return Ok(&self[pos..cur])
            }

            cur += 1;
        }

        Err(SliceReadError::UnexpectedEndOfInput)
    }
}

pub struct SmartFmt<'a> (pub &'a [u8]);

impl<'a> fmt::Display for SmartFmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ascii = true;
        for c in self.0.iter() {
            if *c > 126 || *c < 32 && *c != 0 {
                ascii = false;
                break;
            }
        }

        if ascii {
            write!(f, "\"{}\"", unsafe { str::from_utf8_unchecked(self.0) })
        } else {
            match self.0.len() {
                1 | 2 | 4 | 8 => {
                    try!(write!(f, "0x"));
                    for c in self.0.iter() {
                        try!(write!(f, "{:02x}", c));
                    }
                    Ok(())
                },
                _ => write!(f, "{:?}", self.0)
            }
        }
    }
}

#[macro_export]
macro_rules! trysome {
    ($expr:expr) => (match $expr {
        ::core::result::Result::Ok(val) => val,
        ::core::result::Result::Err(err) => {
            return ::core::option::Option::Some(
                ::core::result::Result::Err(::core::convert::From::from (err))
            )
        }
    })
}
