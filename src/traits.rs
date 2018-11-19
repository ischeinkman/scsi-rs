
use AumsError;

use byteorder::{ByteOrder, BigEndian, LittleEndian};

pub trait Buffer : Sized {
    fn size(&self) -> usize; 
    fn capacity(&self) -> usize; 
    fn is_empty(&self) -> bool {
        self.size() == 0
    }
    fn push_byte(&mut self, byte : u8) -> Result<usize, AumsError> ;
    fn clear(&mut self) -> Result<usize, AumsError> ;
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

pub trait CommunicationChannel {
    fn out_transfer<B : Buffer>(&mut self, bytes : &B) -> Result<usize, AumsError>;
    fn in_transfer<B : Buffer>(&mut self, buffer : &mut B) -> Result<usize, AumsError>;
}