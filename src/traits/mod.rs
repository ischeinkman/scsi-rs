use error::ScsiError;
///
/// The trait that all communication devices should implement if they are to be
/// used to transfer SCSI information.
pub trait CommunicationChannel {
    /// Sends the bytes currently stored in a buffer over the communication channel.
    /// Returns the number of bytes sent.
    fn out_transfer<B: AsRef<[u8]>>(&mut self, bytes: B) -> Result<usize, ScsiError>;

    /// Reads bytes from the channel up to the point where the buffer is filled.
    /// Returns the number of bytes successfully read.
    fn in_transfer<B: AsMut<[u8]>>(&mut self, buffer: B) -> Result<usize, ScsiError>;
}

pub trait BufferPushable {
    fn push_to_buffer<T : AsMut<[u8]>>(&self, buffer : T) -> Result<usize, ScsiError>; 
}

pub trait BufferPullable : Sized {
    fn pull_from_buffer<T : AsRef<[u8]>>(buffer : T) -> Result<Self, ScsiError>;
}