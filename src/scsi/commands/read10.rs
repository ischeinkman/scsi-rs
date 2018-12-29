use scsi::commands::{Command, CommandBlockWrapper, Direction};

use traits::{Buffer, BufferPushable};
use error::{ScsiError, ErrorCause};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Read10Command {
    block_address: u32,
    transfer_bytes: u32,
    transfer_blocks: u16,
}

impl Read10Command {
    pub fn new(
        offset: u32,
        transfer_bytes: u32,
        block_size: u32,
    ) -> Result<Read10Command, ScsiError> {
        let transfer_blocks = if transfer_bytes % block_size != 0 {
            return Err(ScsiError::from_cause(
                ErrorCause::NonBlocksizeMultipleLengthError{actual : transfer_bytes as usize, block_size : block_size as usize},
            ));
        } else {
            (transfer_bytes / block_size) as u16
        };
        if offset % block_size != 0 {
            return Err(ScsiError::from_cause(
                ErrorCause::NonBlocksizeMultipleLengthError{actual : offset as usize, block_size : block_size as usize},
            ));
        }

        Ok(Read10Command {
            block_address : offset / block_size,
            transfer_bytes,
            transfer_blocks,
        })
    }
}

impl BufferPushable for Read10Command {
    fn push_to_buffer<B: Buffer>(&self, buffer: &mut B) -> Result<usize, ScsiError> {
        let mut rval = self.wrapper().push_to_buffer(buffer)?;
        rval += buffer.push_byte(Read10Command::opcode())?;
        rval += buffer.push_byte(0)?;
        rval += buffer.push_u32_be(self.block_address)?;
        rval += buffer.push_byte(0)?;
        rval += buffer.push_u16_be(self.transfer_blocks)?;
        rval += buffer.push_byte(0)?;
        Ok(rval)
    }
}

impl Command for Read10Command {
    fn opcode() -> u8 {
        0x28
    }
    fn length() -> u8 {
        10
    }

    fn wrapper(&self) -> CommandBlockWrapper {
        CommandBlockWrapper::new(
            self.transfer_bytes,
            Direction::IN,
            0,
            Read10Command::length(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Buffer, ScsiError, ErrorCause, Read10Command, BufferPushable};
    pub struct VecNewtype {
        inner : [u8 ; 512], 
        fake_size : usize, 
        read_idx : usize, 
        write_idx : usize, 
    }
    impl VecNewtype {
        pub fn new() -> VecNewtype {
            VecNewtype::with_fake_capacity(512)
        }
        pub fn with_fake_capacity(sz : usize) -> VecNewtype {
            if sz > 512 {
                panic!("Can only use fake vec with max 512 bytes.");
            }
            VecNewtype {
                inner : [0 ; 512], 
                fake_size : sz,
                read_idx : 0, 
                write_idx : 0,
            }
        }
    }
    impl Buffer for VecNewtype {
        fn size(&self) -> usize {
            if self.write_idx >= self.read_idx {
                self.write_idx - self.read_idx
            }
            else {
                (self.fake_size + self.write_idx) - self.read_idx
            }
        }
        fn capacity(&self) -> usize {
            self.fake_size
        }
        fn push_byte(&mut self, byte : u8) -> Result<usize, ScsiError> {
            if self.size() == self.capacity() {
                Err(ScsiError::from_cause(ErrorCause::BufferTooSmallError{expected : 1, actual : 0}))
            }
            else {
                self.inner[self.write_idx] = byte;
                self.write_idx = (self.write_idx + 1) % self.fake_size;
                Ok(1)
            }
        }
        fn pull_byte(&mut self) -> Result<u8, ScsiError> {
            if self.size() == 0 {
                Err(ScsiError::from_cause(ErrorCause::BufferTooSmallError{expected : 1, actual : 0}))
            }
            else {
                let bt = self.inner[self.read_idx];
                self.read_idx = (self.read_idx + 1) % self.fake_size;
                Ok(bt)
            }
        }
    }   

    #[test]
    pub fn test_buffer_push() {
        let expected : [u8 ; 31] = [
            0x55, 0x53, 0x42, 0x43, 
            0x00, 0x00, 0x00, 0x00, 
            0x00, 0x02, 0x00, 0x00,
            0x80, 
            0x00, 
            0x0a, 

            0x28, 
            0x00, 
            0x00, 0x00, 0x12, 0x34, 
            0x00, 
            0x00, 0x01,

            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ];
        let mut buff = VecNewtype::new();
        let read_command = Read10Command::new(0x1234, 512, 512).unwrap();
        assert_eq!(read_command.transfer_blocks, 1);
        let _pushed = read_command.push_to_buffer(&mut buff).unwrap();
        assert_eq!(&buff.inner[0 .. expected.len() as usize], &expected);
    }
}