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

/// Allows a struct to serialize itself to a raw byte buffer.
pub trait BufferPushable {
    /// Serializes `self` to a raw byte slice.
    ///
    /// Returns the number of bytes written on succes.
    ///
    /// #Errors
    ///
    /// Can return a `BufferTooSmall` error when the length of `buffer` is not
    /// large enough to serialize to.
    fn push_to_buffer<T: AsMut<[u8]>>(&self, buffer: T) -> Result<usize, ScsiError>;
}

/// Allows for a struct to deserialize itself from a raw byte buffer.
pub trait BufferPullable: Sized {
    /// Deserializes an instance of `T` from a byte buffer.
    ///
    /// #Errors
    ///
    /// Can return a `BufferTooSmall` error when the length of `buffer` is not
    /// large enough to deserialize from, or a `ParseError` if the buffer
    /// cannot be deserialized into a valid instance of `T` using the bytes
    /// provided.
    fn pull_from_buffer<T: AsRef<[u8]>>(buffer: T) -> Result<Self, ScsiError>;
}
