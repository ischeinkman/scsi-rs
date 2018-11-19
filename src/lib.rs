#![no_std]

extern crate byteorder;
pub mod traits;
pub mod scsi; 

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ErrorCause {
    ParseError, 
    NonBlocksizeMultipleLengthError,
    UsbTransferError,
    FlagError,
    BufferTooSmallError,
    UnsupportedOperationError, 
    InvalidDeviceError, 
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct AumsError {
    cause : ErrorCause,
}

impl AumsError {
    pub fn from_cause(cause : ErrorCause) -> AumsError {
        AumsError {
            cause
        }
    }
}

#[cfg(test)]
mod tests {
    //TODO: Tests
}
