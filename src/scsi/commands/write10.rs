use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{Buffer, BufferPushable};
use error::{ScsiError, ErrorCause};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Write10Command {
    block_address: u32,
    transfer_bytes: u32,
    transfer_blocks: u16,
}

impl Write10Command {
    pub fn new(
        block_address: u32,
        transfer_bytes: u32,
        block_size: u32,
    ) -> Result<Write10Command, ScsiError> {
        if transfer_bytes % block_size != 0 {
            return Err(ScsiError::from_cause(
                ErrorCause::NonBlocksizeMultipleLengthError{actual : transfer_bytes as usize, block_size : block_size as usize},
            ));
        }
        let transfer_blocks = (transfer_bytes / block_size) as u16;
        Ok(Write10Command {
            block_address,
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
