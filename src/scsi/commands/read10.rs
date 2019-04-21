use scsi::commands::{Command, CommandBlockWrapper, Direction};

use error::{ErrorCause, ScsiError};
use traits::{ BufferPullable, BufferPushable};

use byteorder::{ByteOrder, BE};


/// A command to read bytes from the block device. 
/// 
/// The 10 in Read10 (and Write10) stands for the fact that the command is exactly
/// 10 bytes long asside from the BlockWrapper: 
/// 
/// * 1 byte for the opcode
/// * 1 byte padding
/// * 4 bytes for the address of the starting block to read
/// * 1 more padding byte 
/// * 2 bytes for the number of blocks to read 
/// * 1 more padding byte
/// 
/// Due to the need for padding by the specification, Read10 (and Write10) can only
/// deal with a maximum of 65535 blocks per command, which using a standard block size of
/// 512 bytes is only 32 megabytes. This is why the longer 16-byte family of commands were added, 
/// but they are not supported by all devices.  
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Read10Command {
    block_address: u32,
    transfer_bytes: u32,
    transfer_blocks: u16,
}

impl Read10Command {

    /// Creates a new Read10 command. 
    /// 
    /// Executing the created command will attempt to read `transfer_bytes` bytes starting
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
    ) -> Result<Read10Command, ScsiError> {
        let transfer_blocks = if transfer_bytes % block_size != 0 {
            return Err(ScsiError::from_cause(
                ErrorCause::NonBlocksizeMultipleLengthError {
                    actual: transfer_bytes as usize,
                    block_size: block_size as usize,
                },
            ));
        } else {
            (transfer_bytes / block_size) as u16
        };
        if offset % block_size != 0 {
            return Err(ScsiError::from_cause(
                ErrorCause::NonBlocksizeMultipleLengthError {
                    actual: offset as usize,
                    block_size: block_size as usize,
                },
            ));
        }

        Ok(Read10Command {
            block_address: offset / block_size,
            transfer_bytes,
            transfer_blocks,
        })
    }
}

impl BufferPushable for Read10Command {
    fn push_to_buffer<B : AsMut<[u8]>>(&self, mut buffer: B) -> Result<usize, ScsiError> {
        let rval = self.wrapper().push_to_buffer(buffer.as_mut())?;
        let buffer = buffer.as_mut();
        buffer[rval] = Read10Command::opcode();
        buffer[rval + 1] = 0;
        BE::write_u32(&mut buffer[rval + 2..], self.block_address);
        buffer[rval + 6] = 0;
        BE::write_u16(&mut buffer[rval + 7 ..], self.transfer_blocks);
        buffer[rval + 9] = 0;
        Ok(rval + 10)
    }
}

impl BufferPullable for Read10Command {
    fn pull_from_buffer<B : AsRef<[u8]>>(buffer: B) -> Result<Self, ScsiError> {
        let wrapper: CommandBlockWrapper = CommandBlockWrapper::pull_from_buffer(buffer.as_ref())?;
        if wrapper.direction != Direction::IN || wrapper.cb_length != Read10Command::length() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        let buffer = buffer.as_ref();
        let opcode = buffer[15];
        if opcode != Self::opcode() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        let _empty1 = buffer[16];
        let block_address = BE::read_u32(&buffer[17 ..]);
        
        let _empty2 = buffer[21];
        let transfer_blocks = BE::read_u16(&buffer[22 ..]);
        let _empty3 = buffer[24];
        Ok(Read10Command {
            block_address,
            transfer_blocks,
            transfer_bytes: wrapper.data_transfer_length,
        })
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
    use super::Read10Command;
        use crate::{BufferPullable, BufferPushable};

    #[test]
    pub fn test_read10() {
        let expected: [u8; 31] = [
            0x55, 0x53, 0x42, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x80, 0x00,
            0x0a, 0x28, 0x00, 0x00, 0x00, 0x00, 0x8, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];
        let mut buff = [0 ; 32];
        let read_command = Read10Command::new(4096, 512, 512).unwrap();
        assert_eq!(read_command.transfer_blocks, 1);
        let pushed = read_command.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 25);
        assert_eq!(&buff[0..expected.len() as usize], &expected);

        let pulled = Read10Command::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, read_command);
    }
}
