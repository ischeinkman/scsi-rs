/// Abstractions that an environment needs to be able to provide for this
/// crate to use. These traits allow us to abstract away the need to use
/// things like `std`'s `Vec`, `io::Read`, or `io::Write`.
mod buffer;
pub use self::buffer::*;

use error::ScsiError;

///
/// The trait that all communication devices should implement if they are to be
/// used to transfer SCSI information.
pub trait CommunicationChannel {
    /// Sends the bytes currently stored in a buffer over the communication channel.
    /// Returns the number of bytes sent.
    fn out_transfer<B: Buffer>(&mut self, bytes: &mut B) -> Result<usize, ScsiError>;

    /// Reads bytes from the channel up to the point where the buffer is filled.
    /// Returns the number of bytes successfully read.
    fn in_transfer<B: Buffer>(&mut self, buffer: &mut B) -> Result<usize, ScsiError>;
}
