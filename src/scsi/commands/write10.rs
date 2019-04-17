use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{Buffer, BufferPushable, BufferPullable};
use error::{ScsiError, ErrorCause};

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
    block_address: u32,
    transfer_bytes: u32,
    transfer_blocks: u16,
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
            transfer_bytes,
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
            self.transfer_bytes,
            Direction::OUT,
            0,
            Write10Command::length(),
        )
    }
}

impl BufferPushable for Write10Command {
    fn push_to_buffer<B: Buffer>(&self, buffer: &mut B) -> Result<usize, ScsiError> {
        let mut rval = self.wrapper().push_to_buffer(buffer)?;
        rval += buffer.push_byte(Write10Command::opcode())?;
        rval += buffer.push_byte(0)?;
        rval += buffer.push_u32_be(self.block_address)?;
        rval += buffer.push_byte(0)?;
        rval += buffer.push_u16_be(self.transfer_blocks)?;
        rval += buffer.push_byte(0)?;
        Ok(rval)
    }
}

impl BufferPullable for Write10Command {
    fn pull_from_buffer<B : Buffer>(buffer: &mut B) -> Result<Self, ScsiError> {
        let wrapper : CommandBlockWrapper = buffer.pull()?;
        if wrapper.direction != Direction::OUT || wrapper.cb_length != Write10Command::length() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        let opcode_with_padding = buffer.pull_u16_le()?;
        if opcode_with_padding != Write10Command::opcode() as u16 {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        let block_address = buffer.pull_u32_be()?;
        let _padding2 = buffer.pull_byte()?;
        let transfer_blocks = buffer.pull_u16_be()?;
        let _padding3 = buffer.pull_byte()?;
        Ok(Write10Command{ 
            block_address, 
            transfer_blocks, 
            transfer_bytes : wrapper.data_transfer_length
        })
    }
}
#[cfg(test)]
mod tests {
    use super::Write10Command;
    use crate::traits::test::VecNewtype;
    use crate::{BufferPullable, BufferPushable};

    #[test]
    pub fn test_write10() {
        let expected: [u8; 31] = [
            0x55, 0x53, 0x42, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00,
            0x0a, 0x2a, 0x00, 0x00, 0x00, 0x00, 0x8, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];
        let mut buff = VecNewtype::new();
        let read_command = Write10Command::new(4096, 512, 512).unwrap();
        assert_eq!(read_command.transfer_blocks, 1);
        let pushed = read_command.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 25);
        assert_eq!(&buff.inner[0..pushed as usize], &expected[0..pushed]);

        let pulled = Write10Command::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, read_command);
    }
}
