use scsi::commands::{Command, CommandBlockWrapper, Direction};

use traits::{Buffer, BufferPushable};
use error::{ScsiError, ErrorCause};

pub struct Read10Command {
    block_address: u32,
    transfer_bytes: u32,
    transfer_blocks: u16,
}

impl Read10Command {
    pub fn new(
        block_address: u32,
        transfer_bytes: u32,
        block_size: u32,
    ) -> Result<Read10Command, ScsiError> {
        let transfer_blocks = if transfer_bytes % block_size != 0 {
            return Err(ScsiError::from_cause(
                ErrorCause::NonBlocksizeMultipleLengthError,
            ));
        } else {
            (transfer_bytes / block_size) as u16
        };

        Ok(Read10Command {
            block_address,
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
        rval += buffer.push_u16_be(self.transfer_blocks)?;
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
