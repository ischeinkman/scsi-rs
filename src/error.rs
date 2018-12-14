/// A general error struct for the package. 
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ScsiError {
    /// The cause of the error.
    pub cause: ErrorCause,
}

///
/// The cause of a returned error. This is implemented
/// as an enum with a large number of variants to reduce
/// allocations.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ErrorCause {
    /// The error was caused because the library failed to parse a byte buffer
    /// into a struct correctly.
    ParseError,

    /// The error was caused because a passed-in byte length was not a multiple
    /// of the previously provided block size.
    NonBlocksizeMultipleLengthError{actual : usize, block_size : usize},

    /// The error was caused because we failed to read/write enough bytes to/from
    /// the communication device.
    UsbTransferError{direction : UsbTransferDirection},

    /// The error was thrown because a struct's flags were invalid.
    FlagError {flags : u32},

    /// The error was thrown becaused we attempted to read/write too many bytes
    /// from/to a buffer.
    BufferTooSmallError{expected : usize, actual : usize},

    /// The error was thrown because we called a method for a trait that
    /// isn't valid in this particular struct's implementation.
    UnsupportedOperationError,

    /// The error was thrown because we tried connecting to a device we don't support.
    InvalidDeviceError,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum UsbTransferDirection {
    In, 
    Out, 
}

impl ScsiError {
    /// Constructs a new ScsiError struct from a particular cause.
    pub fn from_cause(cause: ErrorCause) -> ScsiError {
        ScsiError { cause }
    }
}