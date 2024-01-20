/// Convenience alias for the [`Result`](core::result::Result) type.
pub type Result<T> = core::result::Result<T, Error>;

pub type SliceReadResult<T> = core::result::Result<T, SliceReadError>;

pub type VecWriteResult = core::result::Result<(), VecWriteError>;

/// An error describe parsing problems when creating device trees.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The magic number `MAGIC_NUMBER` was not found at the start of the
    /// structure.
    InvalidMagicNumber,

    /// An offset or size found inside the device tree is outside of what was
    /// supplied to `load()`.
    SizeMismatch,

    /// Failed to read data from slice.
    SliceReadError(SliceReadError),

    /// The data format was not as expected at the given position
    ParseError(usize),

    /// While trying to convert a string that was supposed to be ASCII, invalid
    /// utf8 sequences were encounted
    Utf8Error,

    /// The device tree version is not supported by this library.
    VersionNotSupported,

    /// The device tree structure could not be serialized to DTB
    VecWriteError(VecWriteError),

    /// Property could not be parsed
    PropError(PropError),
}

impl From<SliceReadError> for Error {
    fn from(e: SliceReadError) -> Error {
        Error::SliceReadError(e)
    }
}

impl From<VecWriteError> for Error {
    fn from(e: VecWriteError) -> Error {
        Error::VecWriteError(e)
    }
}

impl From<PropError> for Error {
    fn from(e: PropError) -> Self {
        Self::PropError(e)
    }
}

impl From<core::str::Utf8Error> for Error {
    fn from(_: core::str::Utf8Error) -> Error {
        Error::Utf8Error
    }
}

/// Represents property errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropError {
    NotFound,
    Utf8Error,
    Missing0,
    SliceReadError(SliceReadError),
}

impl From<core::str::Utf8Error> for PropError {
    fn from(_: core::str::Utf8Error) -> PropError {
        PropError::Utf8Error
    }
}

impl From<SliceReadError> for PropError {
    fn from(e: SliceReadError) -> PropError {
        PropError::SliceReadError(e)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SliceReadError {
    UnexpectedEndOfInput,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VecWriteError {
    NonContiguousWrite,
    UnalignedWrite,
}
