
use AumsError;

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
}

pub trait BufferPushable {
    fn push_to_buffer<B : Buffer>(&self, buffer : &mut B) -> Result<usize, AumsError>;
}

pub trait BufferPullable : Sized{
    fn pull_from_buffer<B : Buffer>(buffer : &mut B) -> Result<Self, AumsError>;
}

impl BufferPushable for u8 {
    fn push_to_buffer<B : Buffer>(&self, buffer : &mut B) -> Result<usize, AumsError> {
        buffer.push_byte(*self)
    }
}

impl BufferPullable for u8 {
    fn pull_from_buffer<B : Buffer>(buffer : &mut B) -> Result<u8, AumsError> {
        buffer.pull_byte()
    }
}

impl BufferPushable for u16 {
    fn push_to_buffer<B : Buffer>(&self, buffer : &mut B) -> Result<usize, AumsError> { 
        let first_byte = ((*self & 0xFF00) >> 8) as u8;
        let second_byte = (*self & 0x00FF) as u8;

        let rval = buffer.push_byte(first_byte)? + buffer.push_byte(second_byte)?;
        Ok(rval)
    }
}

impl BufferPullable for u16 {
    fn pull_from_buffer<B : Buffer>(buffer : &mut B) -> Result<u16, AumsError> {
        let first_byte = buffer.pull_byte()?;
        let second_byte = buffer.pull_byte()?;
        let rval = (first_byte as u16) << 8 | (second_byte as u16);
        Ok(rval)
    }
}

impl BufferPushable for u32 {
    fn push_to_buffer<B : Buffer>(&self, buffer : &mut B) -> Result<usize, AumsError> { 
        let first_half = ((*self & 0xFFFF0000) >> 16) as u16; 
        let second_half = ((*self & 0x0000FFFF)) as u16;
        let rval = first_half.push_to_buffer(buffer)? + second_half.push_to_buffer(buffer)?;
        Ok(rval)
    }
}

impl BufferPullable for u32 {
    fn pull_from_buffer<B : Buffer>(buffer : &mut B) -> Result<u32, AumsError> {
        let first_half = u16::pull_from_buffer(buffer)?;
        let second_half = u16::pull_from_buffer(buffer)?;
        let rval = (first_half as u32) << 16 | (second_half as u32);
        Ok(rval)
    }
}

impl BufferPushable for u64 {
    fn push_to_buffer<B : Buffer>(&self, buffer : &mut B) -> Result<usize, AumsError> { 
        let first_half = ((*self & 0xFFFFFFFF00000000) >> 32) as u32; 
        let second_half = ((*self & 0x00000000FFFFFFFF)) as u32;
        let rval = first_half.push_to_buffer(buffer)? + second_half.push_to_buffer(buffer)?;
        Ok(rval)
    }
}

impl BufferPullable for u64 {
    fn pull_from_buffer<B : Buffer>(buffer : &mut B) -> Result<u64, AumsError> {
        let first_half = u32::pull_from_buffer(buffer)?;
        let second_half = u32::pull_from_buffer(buffer)?;
        let rval = (first_half as u64) << 32 | (second_half as u64);
        Ok(rval)
    }
}


pub trait CommunicationChannel {
    fn out_transfer<B : Buffer>(&mut self, bytes : &B) -> Result<usize, AumsError>;
    fn in_transfer<B : Buffer>(&mut self, buffer : &mut B) -> Result<usize, AumsError>;
}