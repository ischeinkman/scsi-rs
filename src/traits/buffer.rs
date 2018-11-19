use error::{ErrorCause, ScsiError};

use byteorder::{BigEndian, ByteOrder, LittleEndian};

/// A trait to provide an abstraction over a FIFO byte buffer.
///  
/// The buffer is modeled with 2 different indices that operate
/// independently: one that stores the location of the next byte to be read from,
/// and one storeing the location of the next byte to write to.
pub trait Buffer: Sized {
    /// The total number of bytes written to this buffer; in the case of a slice-
    /// backed buffer, this would be equal to the write index.
    fn size(&self) -> usize;

    /// The maximum number of bytes possible to store in this buffer.
    fn capacity(&self) -> usize;

    /// Checks whether or not the buffer is empty. By default this is implemented
    /// via the buffer's size.
    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// Adds a byte to the end of the buffer, incrementing the write index.
    /// Returns Ok(1) on success.
    fn push_byte(&mut self, byte: u8) -> Result<usize, ScsiError>;

    /// Retrieves a byte from the beginning of the buffer, incrementing the read
    /// index.
    /// Returns Ok(1) on success.
    fn pull_byte(&mut self) -> Result<u8, ScsiError>;

    /// Retrieves a struct from the beginning of the buffer using the struct's
    /// pull_from_buffer method.
    /// Returns the parsed struct.
    fn pull<T: BufferPullable>(&mut self) -> Result<T, ScsiError> {
        T::pull_from_buffer(self)
    }

    /// Resets the read index to the beginning of the buffer.
    fn reset_read_head(&mut self);

    /// Resets the write index to the beginning of the buffer.
    fn reset_write_head(&mut self);

    /// Tries to expand the buffer by the given length.
    /// On success, should return an instance of Self with the following
    /// facts being true:
    /// *  The returned buffer's read index and write index are the same as the
    ///    original buffer's.
    ///
    /// *  All bytes from the beginning of the buffer until the byte before the
    ///    write index match those from the original buffer.
    ///
    /// *  The returned buffer's capacity = the original buffer's capacity + bytes
    ///
    /// If it is not possible to create a buffer matching these properties, an
    /// error is returned.
    fn expand_by(self, bytes: usize) -> Result<Self, ScsiError>;

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

/// A wrapper struct that implements the `Buffer` trait around a backing `&mut [u8]`.
pub struct SliceBuffer<'a> {
    read_idx: usize,
    write_idx: usize,
    inner: &'a mut [u8],
}

impl<'a> SliceBuffer<'a> {
    /// Wraps a given slice in a buffer with a read index and write index at the
    /// beginning of the slice.
    pub fn new(slice: &'a mut [u8]) -> SliceBuffer<'a> {
        SliceBuffer {
            read_idx: 0,
            write_idx: 0,
            inner: slice,
        }
    }

    /// Unwraps the buffer, retrieving its backing slice.
    pub fn into_inner(self) -> &'a mut [u8] {
        self.inner
    }
}
impl<'a> Buffer for SliceBuffer<'a> {
    fn size(&self) -> usize {
        self.write_idx
    }
    fn capacity(&self) -> usize {
        self.inner.len()
    }

    fn push_byte(&mut self, byte: u8) -> Result<usize, ScsiError> {
        if self.write_idx >= self.inner.len() {
            Err(ScsiError::from_cause(ErrorCause::BufferTooSmallError))
        } else {
            self.inner[self.write_idx] = byte;
            self.write_idx += 1;
            Ok(1)
        }
    }
    fn pull_byte(&mut self) -> Result<u8, ScsiError> {
        if self.read_idx >= self.write_idx {
            Err(ScsiError::from_cause(ErrorCause::BufferTooSmallError))
        } else {
            let bt = self.inner[self.read_idx];
            self.read_idx += 1;
            Ok(bt)
        }
    }

    fn reset_read_head(&mut self) {
        self.read_idx = 0;
    }
    fn reset_write_head(&mut self) {
        self.write_idx = 0;
    }
    fn expand_by(self, _bytes: usize) -> Result<Self, ScsiError> {
        Err(ScsiError::from_cause(ErrorCause::UnsupportedOperationError))
    }
}
