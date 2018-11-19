
use AumsError;
use ErrorCause;

use byteorder::{ByteOrder, BigEndian, LittleEndian};

pub trait Buffer : Sized {
    fn size(&self) -> usize; 
    fn capacity(&self) -> usize; 
    fn is_empty(&self) -> bool {
        self.size() == 0
    }
    fn push_byte(&mut self, byte : u8) -> Result<usize, AumsError> ;
    fn pull_byte(&mut self) -> Result<u8, AumsError>;
    fn pull<T : BufferPullable>(&mut self) -> Result<T, AumsError> {
        T::pull_from_buffer(self)
    }
    fn reset_read_head(&mut self);
    fn reset_write_head(&mut self);
    fn expand_by(self, bytes : usize) -> Result<Self, AumsError> ;


    fn pull_u16_le(&mut self) -> Result<u16, AumsError> {
        const BYTES : usize = 2;
        type ED = LittleEndian;

        let mut bytes = [0 ; BYTES];
        for idx in 0 .. BYTES {
            bytes[idx] = self.pull_byte()?;
        }
        Ok(ED::read_u16(&bytes))
    }
    fn pull_u16_be(&mut self) -> Result<u16, AumsError> {
        const BYTES : usize = 2;
        type ED = BigEndian;

        let mut bytes = [0 ; BYTES];
        for idx in 0 .. BYTES {
            bytes[idx] = self.pull_byte()?;
        }
        Ok(ED::read_u16(&bytes))
    }
    fn pull_u32_le(&mut self) -> Result<u32, AumsError> {
        const BYTES : usize = 4;
        type ED = LittleEndian;

        let mut bytes = [0 ; BYTES];
        for idx in 0 .. BYTES {
            bytes[idx] = self.pull_byte()?;
        }
        Ok(ED::read_u32(&bytes))
    }
    
    fn pull_u32_be(&mut self) -> Result<u32, AumsError> {
        const BYTES : usize = 4;
        type ED = BigEndian;

        let mut bytes = [0 ; BYTES];
        for idx in 0 .. BYTES {
            bytes[idx] = self.pull_byte()?;
        }
        Ok(ED::read_u32(&bytes))
    }
    fn pull_u64_le(&mut self) -> Result<u64, AumsError> {
        const BYTES : usize = 8;
        type ED = LittleEndian;

        let mut bytes = [0 ; BYTES];
        for idx in 0 .. BYTES {
            bytes[idx] = self.pull_byte()?;
        }
        Ok(ED::read_u64(&bytes))
    }
    
    fn pull_u64_be(&mut self) -> Result<u64, AumsError> {
        const BYTES : usize = 8;
        type ED = BigEndian;

        let mut bytes = [0 ; BYTES];
        for idx in 0 .. BYTES {
            bytes[idx] = self.pull_byte()?;
        }
        Ok(ED::read_u64(&bytes))
    }
    
    fn push_u16_le(&mut self, n : u16) -> Result<usize, AumsError> {
        const BYTES : usize = 2;
        type ED = LittleEndian;

        let mut bytes = [0 ; BYTES];
        ED::write_u16(&mut bytes, n);
        bytes.push_to_buffer(self)?;
        Ok(BYTES)
    }
    fn push_u16_be(&mut self, n : u16) -> Result<usize, AumsError> {
        const BYTES : usize = 2;
        type ED = BigEndian;

        let mut bytes = [0 ; BYTES];
        ED::write_u16(&mut bytes, n);
        bytes.push_to_buffer(self)?;
        Ok(BYTES)
    }
    fn push_u32_le(&mut self, n : u32) -> Result<usize, AumsError> {
        const BYTES : usize = 4;
        type ED = LittleEndian;

        let mut bytes = [0 ; BYTES];
        ED::write_u32(&mut bytes, n);
        bytes.push_to_buffer(self)?;
        Ok(BYTES)
    }
    fn push_u32_be(&mut self, n : u32) -> Result<usize, AumsError> {
        const BYTES : usize = 4;
        type ED = BigEndian;

        let mut bytes = [0 ; BYTES];
        ED::write_u32(&mut bytes, n);
        bytes.push_to_buffer(self)?;
        Ok(BYTES)
    }
    fn push_u64_le(&mut self, n : u64) -> Result<usize, AumsError> {
        const BYTES : usize = 8;
        type ED = LittleEndian;

        let mut bytes = [0 ; BYTES];
        ED::write_u64(&mut bytes, n);
        bytes.push_to_buffer(self)?;
        Ok(BYTES)
    }
    fn push_u64_be(&mut self, n : u64) -> Result<usize, AumsError> {
        const BYTES : usize = 8;
        type ED = BigEndian;

        let mut bytes = [0 ; BYTES];
        ED::write_u64(&mut bytes, n);
        bytes.push_to_buffer(self)?;
        Ok(BYTES)
    }
}

pub trait BufferPushable {
    fn push_to_buffer<B : Buffer>(&self, buffer : &mut B) -> Result<usize, AumsError>;
}

impl BufferPushable for u8 {
    fn push_to_buffer<B : Buffer>(&self, buffer : &mut B) -> Result<usize, AumsError> {
        buffer.push_byte(*self)
    }
}

impl <'a, T> BufferPushable for &'a [T] where T : BufferPushable {
    fn push_to_buffer<B : Buffer>(&self, buffer : &mut B) -> Result<usize, AumsError> {
        let mut rval = 0;
        for itm in self.iter() {
            rval += itm.push_to_buffer(buffer)?;
        }
        Ok(rval)
    }
}

impl <T> BufferPushable for [T] where T : BufferPushable {
    fn push_to_buffer<B : Buffer>(&self, buffer : &mut B) -> Result<usize, AumsError> {
        let mut rval = 0;
        for itm in self.iter() {
            rval += itm.push_to_buffer(buffer)?;
        }
        Ok(rval)
    }
}

pub trait BufferPullable : Sized{
    fn pull_from_buffer<B : Buffer>(buffer : &mut B) -> Result<Self, AumsError>;
}


pub struct SliceBuffer<'a> {
    read_idx : usize, 
    write_idx : usize, 
    inner : &'a mut [u8]
}

impl <'a> SliceBuffer<'a> {
    pub fn new(slice : &'a mut [u8]) -> SliceBuffer<'a> {
        SliceBuffer {
            read_idx : 0, 
            write_idx : 0, 
            inner : slice
        }
    }

    pub fn into_inner(self) -> &'a mut [u8] {
        self.inner
    }
}
impl <'a> Buffer for SliceBuffer<'a> {
    fn size(&self) -> usize {
        self.write_idx
    }
    fn capacity(&self) -> usize {
        self.inner.len()
    }

    fn push_byte(&mut self, byte : u8) -> Result<usize, AumsError> {
        if self.write_idx >= self.inner.len() {
            Err(AumsError::from_cause(ErrorCause::BufferTooSmallError))
        }
        else {
            self.inner[self.write_idx] = byte;
            self.write_idx += 1;
            Ok(1)
        }
    }
    fn pull_byte(&mut self) -> Result<u8, AumsError> {
        if self.read_idx >= self.write_idx {
            Err(AumsError::from_cause(ErrorCause::BufferTooSmallError))
        }
        else {
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
    fn expand_by(self, _bytes : usize) -> Result<Self, AumsError> {
        Err(AumsError::from_cause(ErrorCause::UnsupportedOperationError))
    }
}