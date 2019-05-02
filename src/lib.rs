
#![cfg_attr(not(test), no_std)]
extern crate byteorder;
pub mod scsi;
mod traits;
mod error;

pub use error::*;
pub use traits::*;

#[cfg(test)]
mod tests {
    //TODO: Tests
}
