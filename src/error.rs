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
    NonBlocksizeMultipleLengthError {
        /// The transfer length that was passed in, but was rejected because
        /// `actual % block_size != 0`.
        actual: usize,

        /// The expected block size.
        block_size: usize,
    },

    /// The error was caused because we failed to read/write enough bytes to/from
    /// the communication device.
    UsbTransferError {
        /// The direction the failed transfer was in.
        direction: UsbTransferDirection,
    },

    /// The error was thrown because a struct's flags were invalid.
    FlagError {
        /// The value of whatever flag that was rejected.
        flags: u32,
    },

    /// The error was thrown becaused we attempted to read/write too many bytes
    /// from/to a buffer.
    BufferTooSmallError {
        /// The size of the buffer the read/write expected.
        expected: usize,

        /// The actual size of the buffer.
        actual: usize,
    },

    /// The error was thrown because we called a method for a trait that
    /// isn't valid in this particular struct's implementation.
    UnsupportedOperationError,

    /// The error was thrown because we tried connecting to a device we don't support.
    InvalidDeviceError,
}

/// The direction that the USB transfer was going when it errored.
///
/// Directions are relative to whatever central SCSI struct is being used at the
/// time, either an `ScsiBlockDevice` or `ScsiResponder`.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum UsbTransferDirection {
    /// The USB transfer was from the other device to ourselves.
    In,

    /// The transfer was from ourselves to the other device.
    Out,
}

impl ScsiError {
    /// Constructs a new ScsiError struct from a particular cause.
    pub fn from_cause(cause: ErrorCause) -> ScsiError {
        ScsiError { cause }
    }
}
