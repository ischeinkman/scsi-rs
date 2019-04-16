use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{Buffer, BufferPushable, BufferPullable};
use error::{ScsiError, ErrorCause};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Write10Command {
    block_address: u32,
    transfer_bytes: u32,
    transfer_blocks: u16,
}

impl Write10Command {
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
        Ok(Write10Command{ 
            block_address, 
            transfer_blocks, 
            transfer_bytes : wrapper.data_transfer_length
        })
    }
}