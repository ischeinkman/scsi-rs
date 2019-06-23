//! A crate for both interacting with SCSI devices as a host and for writing
//! new SCSI device software/firmware.
//!
//! Currently the main focus of this crate is Bulk Only USB Mass Storage Device
//! compatibility, since that comprises a significant chunk of use cases. However,
//! more functionality can be requested and/or PRed as necessary or desired.

#![warn(missing_docs)]
#![cfg_attr(not(test), no_std)]
extern crate byteorder;
mod error;
pub mod scsi;
mod traits;

pub use error::*;
pub use traits::*;
