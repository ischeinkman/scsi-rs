//! Contains implementations for the different SCSI commands and responses, 
//! as well as helper utilities to, for example, treat the SCSI device as a 
//! normal block devies. 


/// Contains implementations of the different SCSI commands and responses. 
pub mod commands;

mod device;
pub use self::device::*;
