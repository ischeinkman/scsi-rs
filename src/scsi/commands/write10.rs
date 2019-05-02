use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{BufferPushable, BufferPullable};
use error::{ScsiError, ErrorCause};

use byteorder::{ByteOrder, BE};

/// A command to write bytes to the block device. 
/// 
/// The 10 in Write10 (and Read10) stands for the fact that the command is exactly
/// 10 bytes long asside from the BlockWrapper: 
/// 
/// * 1 byte for the opcode
/// * 1 byte padding
/// * 4 bytes for the address of the starting block to write to
/// * 1 more padding byte 
/// * 2 bytes for the number of blocks to write 
/// * 1 more padding byte
/// 
/// Due to the need for padding by the specification, Write10 (and Read10) can only
/// deal with a maximum of 65535 blocks per command, which using a standard block size of
/// 512 bytes is only 32 megabytes. This is why the longer 16-byte family of commands were added, 
/// but they are not supported by all devices.  
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Write10Command {
    pub block_address: u32,
    pub block_size: u32,
    pub transfer_blocks: u16,
}

impl Write10Command {
    
    /// Creates a new Write10 command. 
    /// 
    /// Executing the created command will attempt to write `transfer_bytes` bytes starting
    /// at `offset` bytes from the head of an SCSI device which has a block size of `block_size`. 
    /// 
    /// # Errors
    /// This function returns an error if either `offset` or `transfer_bytes` are 
    /// not an integer multiple of `block_size`, since SCSI cannot address at lower 
    /// than block resolution. 
    pub fn new(
        offset: u32,
        transfer_bytes: u32,
        block_size: u32,
    ) -> Result<Write10Command, ScsiError> {
        if transfer_bytes == 0 || transfer_bytes % block_size != 0 {
            return Err(ScsiError::from_cause(
                ErrorCause::NonBlocksizeMultipleLengthError{actual : transfer_bytes as usize, block_size : block_size as usize},
            ));
        }
        let transfer_blocks = (transfer_bytes / block_size) as u16;
        if offset % block_size != 0 {
            return Err(ScsiError::from_cause(
                ErrorCause::NonBlocksizeMultipleLengthError{actual : offset as usize, block_size : block_size as usize},
            ));
        }
        Ok(Write10Command {
            block_address : offset / block_size,
            block_size,
            transfer_blocks,
        })
    }
}

impl Command for Write10Command {
    fn opcode() -> u8 {
        0x2a
    }
    fn length() -> u8 {
        10
    }
    fn wrapper(&self) -> CommandBlockWrapper {
        CommandBlockWrapper::new(
            self.block_size * (self.transfer_blocks as u32),
            Direction::OUT,
            0,
            Write10Command::length(),
        )
    }
}

impl BufferPushable for Write10Command {
    fn push_to_buffer<B: AsMut<[u8]>>(&self, mut buffer: B) -> Result<usize, ScsiError> {
        let mut buffer = buffer.as_mut();
        let rval = self.wrapper().push_to_buffer(&mut buffer)?;
        buffer[rval] = Write10Command::opcode();
        buffer[rval + 1] = 0;
        BE::write_u32(&mut buffer[rval + 2 ..], self.block_address);
        buffer[rval + 6] = 0;
        BE::write_u16(&mut buffer[rval + 7 ..], self.transfer_blocks);
        buffer[rval + 9] = 0;
        Ok(rval + 10)
    }
}

impl BufferPullable for Write10Command {
    fn pull_from_buffer<B : AsRef<[u8]>>(buffer: B) -> Result<Self, ScsiError> {
        let wrapper : CommandBlockWrapper = CommandBlockWrapper::pull_from_buffer(buffer.as_ref())?;
        let buffer = &buffer.as_ref()[15 ..];
        if wrapper.direction != Direction::OUT || wrapper.cb_length != Write10Command::length() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        let opcode = buffer[0];
        if opcode != Write10Command::opcode() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        let block_address = BE::read_u32(&buffer[2 ..]);
        let transfer_blocks = BE::read_u16(&buffer[7 ..]);
        Ok(Write10Command{ 
            block_address, 
            transfer_blocks, 
            block_size : wrapper.data_transfer_length/(transfer_blocks as u32)
        })
    }
}
#[cfg(test)]
mod tests {
    use super::Write10Command;
    use crate::{BufferPullable, BufferPushable};

    #[test]
    pub fn test_write10() {
        let expected: [u8; 31] = [
            0x55, 0x53, 0x42, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00,
            0x0a, 0x2a, 0x00, 0x00, 0x00, 0x00, 0x8, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];
        let mut buff = [0 ; 32];
        let read_command = Write10Command::new(4096, 512, 512).unwrap();
        assert_eq!(read_command.transfer_blocks, 1);
        let pushed = read_command.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 25);
        assert_eq!(&buff[0..pushed as usize], &expected[0..pushed]);

        let pulled = Write10Command::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, read_command);
    }
}
