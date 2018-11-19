#![no_std]

extern crate byteorder;
pub mod traits;
pub mod scsi; 

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ErrorCause {
    ParseError, 
    OutOfInputError, 
    InvalidInputError,
    UsbTransferError,
    FlagError, 
    Other,
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
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
