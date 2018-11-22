use error::{ErrorCause, ScsiError};

use byteorder::{BigEndian, ByteOrder, LittleEndian};

/// A trait to provide an abstraction over an arbitrary byte buffer.
///
/// By using this trait, we can still perform "pseudo-allocations" without
/// requiring a literal system allocation. Semantically speaking this is a
/// fixed-size FIFO "queue" of bytes with a variety of helper methods to
/// work with the buffer itself easily.
pub trait Buffer: Sized {
    /// How many bytes are currently stored in the buffer.
    fn size(&self) -> usize;

    /// The maximum number of bytes possible to store in this buffer.
    fn capacity(&self) -> usize;

    /// Checks whether or not the buffer is empty. By default this is implemented
    /// via the buffer's size.
    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// Checks whether or not the buffer is full.
    fn is_full(&self) -> bool {
        self.capacity() == self.size()
    }

    /// Pushes a byte to the end of the buffer.
    ///
    /// Returns `Ok(1)` on succes.
    fn push_byte(&mut self, byte: u8) -> Result<usize, ScsiError>;

    /// Retrieves a byte from the beginning of the buffer.
    ///
    /// Returns Ok(1) on success.
    fn pull_byte(&mut self) -> Result<u8, ScsiError>;

    /// Retrieves a struct `T` from the beginning of the buffer via
    /// `T::pull_from_buffer`.
    ///
    /// Returns the parsed struct.
    fn pull<T: BufferPullable>(&mut self) -> Result<T, ScsiError> {
        T::pull_from_buffer(self)
    }

    /// Pulls a `u16` from the beginning of the buffer in Little Endian.
    fn pull_u16_le(&mut self) -> Result<u16, ScsiError> {
        const BYTES: usize = 2;
        type ED = LittleEndian;

        let mut bytes = [0; BYTES];
        for bt in bytes.iter_mut() {
            *bt = self.pull_byte()?;
        }
        Ok(ED::read_u16(&bytes))
    }

    /// Pulls a `u16` from the beginning of the buffer in Big Endian.
    fn pull_u16_be(&mut self) -> Result<u16, ScsiError> {
        const BYTES: usize = 2;
        type ED = BigEndian;

        let mut bytes = [0; BYTES];
        for bt in bytes.iter_mut() {
            *bt = self.pull_byte()?;
        }
        Ok(ED::read_u16(&bytes))
    }

    /// Pulls a `u32` from the beginning of the buffer in Little Endian.
    fn pull_u32_le(&mut self) -> Result<u32, ScsiError> {
        const BYTES: usize = 4;
        type ED = LittleEndian;

        let mut bytes = [0; BYTES];
        for bt in bytes.iter_mut() {
            *bt = self.pull_byte()?;
        }
        Ok(ED::read_u32(&bytes))
    }

    /// Pulls a `u32` from the beginning of the buffer in Big Endian.
    fn pull_u32_be(&mut self) -> Result<u32, ScsiError> {
        const BYTES: usize = 4;
        type ED = BigEndian;

        let mut bytes = [0; BYTES];
        for bt in bytes.iter_mut() {
            *bt = self.pull_byte()?;
        }
        Ok(ED::read_u32(&bytes))
    }

    /// Pulls a `u32` from the beginning of the buffer in Little Endian.
    fn pull_u64_le(&mut self) -> Result<u64, ScsiError> {
        const BYTES: usize = 8;
        type ED = LittleEndian;

        let mut bytes = [0; BYTES];
        for bt in bytes.iter_mut() {
            *bt = self.pull_byte()?;
        }
        Ok(ED::read_u64(&bytes))
    }

    /// Pulls a `u64` from the beginning of the buffer in Big Endian.
    fn pull_u64_be(&mut self) -> Result<u64, ScsiError> {
        const BYTES: usize = 8;
        type ED = BigEndian;

        let mut bytes = [0; BYTES];
        for bt in bytes.iter_mut() {
            *bt = self.pull_byte()?;
        }
        Ok(ED::read_u64(&bytes))
    }

    /// Pushes a `u16` to the end of the buffer in Little Endian.  
    fn push_u16_le(&mut self, n: u16) -> Result<usize, ScsiError> {
        const BYTES: usize = 2;
        type ED = LittleEndian;

        let mut bytes = [0; BYTES];
        ED::write_u16(&mut bytes, n);
        bytes.push_to_buffer(self)?;
        Ok(BYTES)
    }

    /// Pushes a `u16` to the end of the buffer in Big Endian.  
    fn push_u16_be(&mut self, n: u16) -> Result<usize, ScsiError> {
        const BYTES: usize = 2;
        type ED = BigEndian;

        let mut bytes = [0; BYTES];
        ED::write_u16(&mut bytes, n);
        bytes.push_to_buffer(self)?;
        Ok(BYTES)
    }
    /// Pushes a `u32` to the end of the buffer in Little Endian.  
    fn push_u32_le(&mut self, n: u32) -> Result<usize, ScsiError> {
        const BYTES: usize = 4;
        type ED = LittleEndian;

        let mut bytes = [0; BYTES];
        ED::write_u32(&mut bytes, n);
        bytes.push_to_buffer(self)?;
        Ok(BYTES)
    }
    /// Pushes a `u32` to the end of the buffer in Big Endian.  
    fn push_u32_be(&mut self, n: u32) -> Result<usize, ScsiError> {
        const BYTES: usize = 4;
        type ED = BigEndian;

        let mut bytes = [0; BYTES];
        ED::write_u32(&mut bytes, n);
        bytes.push_to_buffer(self)?;
        Ok(BYTES)
    }
    /// Pushes a `u64` to the end of the buffer in Little Endian.  
    fn push_u64_le(&mut self, n: u64) -> Result<usize, ScsiError> {
        const BYTES: usize = 8;
        type ED = LittleEndian;

        let mut bytes = [0; BYTES];
        ED::write_u64(&mut bytes, n);
        bytes.push_to_buffer(self)?;
        Ok(BYTES)
    }
    /// Pushes a `u64` to the end of the buffer in Big Endian.  
    fn push_u64_be(&mut self, n: u64) -> Result<usize, ScsiError> {
        const BYTES: usize = 8;
        type ED = BigEndian;

        let mut bytes = [0; BYTES];
        ED::write_u64(&mut bytes, n);
        bytes.push_to_buffer(self)?;
        Ok(BYTES)
    }
}

/// A trait to represent a struct that can be pushed to a byte buffer in a constant,
/// predetermined way.
pub trait BufferPushable {
    /// Pushes `self` to a buffer, returning the number of bytes pushed on success.
    fn push_to_buffer<B: Buffer>(&self, buffer: &mut B) -> Result<usize, ScsiError>;
}

impl BufferPushable for u8 {
    fn push_to_buffer<B: Buffer>(&self, buffer: &mut B) -> Result<usize, ScsiError> {
        buffer.push_byte(*self)
    }
}

impl<'a, T> BufferPushable for &'a [T]
where
    T: BufferPushable,
{
    fn push_to_buffer<B: Buffer>(&self, buffer: &mut B) -> Result<usize, ScsiError> {
        let mut rval = 0;
        for itm in self.iter() {
            rval += itm.push_to_buffer(buffer)?;
        }
        Ok(rval)
    }
}

impl<T> BufferPushable for [T]
where
    T: BufferPushable,
{
    fn push_to_buffer<B: Buffer>(&self, buffer: &mut B) -> Result<usize, ScsiError> {
        let mut rval = 0;
        for itm in self.iter() {
            rval += itm.push_to_buffer(buffer)?;
        }
        Ok(rval)
    }
}

/// A trait to represent a struct that can be pulled from a byte buffer in a constant,
/// predetermined way.
pub trait BufferPullable: Sized {
    /// Pulls an instance from the given buffer, returning the new instance on success.
    fn pull_from_buffer<B: Buffer>(buffer: &mut B) -> Result<Self, ScsiError>;
}

/// A struct to wrap a byte-slice-convertable struct into a fixed-size buffer
/// that wraps its boundaries as necessary.
pub struct SliceBuffer<T: AsMut<[u8]> + AsRef<[u8]>> {
    read_idx: usize,
    write_idx: usize,
    inner: T,
}

impl<T: AsMut<[u8]> + AsRef<[u8]>> SliceBuffer<T> {
    /// Wraps a given slice in a buffer with a read index and write index at the
    /// beginning of the slice.
    pub fn new(slice: T) -> SliceBuffer<T> {
        SliceBuffer {
            read_idx: 0,
            write_idx: 0,
            inner: slice,
        }
    }

    /// Unwraps the buffer, retrieving its backing object.
    pub fn into_inner(self) -> T {
        self.inner
    }
}
impl<T: AsMut<[u8]> + AsRef<[u8]>> Buffer for SliceBuffer<T> {
    fn size(&self) -> usize {
        if self.read_idx == self.write_idx {
            self.inner.as_ref().len()
        } else if self.write_idx > self.read_idx {
            self.write_idx - self.read_idx
        } else {
            self.inner.as_ref().len() - self.read_idx + self.write_idx
        }
    }
    fn capacity(&self) -> usize {
        self.inner.as_ref().len()
    }
    fn is_full(&self) -> bool {
        self.write_idx == self.read_idx
    }
    fn push_byte(&mut self, byte: u8) -> Result<usize, ScsiError> {
        if self.is_full() {
            Err(ScsiError::from_cause(ErrorCause::BufferTooSmallError {expected : 1, actual : 0}))
        } else {
            self.inner.as_mut()[self.write_idx] = byte;
            self.write_idx = (self.write_idx + 1) % self.inner.as_mut().len();
            Ok(1)
        }
    }
    fn pull_byte(&mut self) -> Result<u8, ScsiError> {
        if self.is_empty() {
            Err(ScsiError::from_cause(ErrorCause::BufferTooSmallError{expected : 1, actual : 0}))
        } else {
            let byte = self.inner.as_mut()[self.read_idx];
            self.read_idx = (self.read_idx + 1) % self.inner.as_ref().len();
            Ok(byte)
        }
    }
}
