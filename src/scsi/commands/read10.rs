
use scsi::commands::{CommmandBlockWrapper, Command, Direction};

use traits::{Buffer, BufferPushable};
use crate::{AumsError, ErrorCause};

pub struct Read10Command {
    wrapper : CommmandBlockWrapper,
    block_address : u32, 
    _transfer_bytes : u32, 
    _block_size : u32, 
    transfer_blocks : u16
}

impl Read10Command {
    pub fn new(block_address : u32, transfer_bytes : u32, block_size : u32) -> Result<Read10Command, AumsError> {
        let transfer_blocks = if transfer_bytes % block_size != 0 {
            return Err(AumsError::from_cause(ErrorCause::InvalidInputError));
        } else {(transfer_bytes / block_size) as u16};

        let wrapper = CommmandBlockWrapper::new(0, Direction::IN, 0, Read10Command::length());
        Ok(Read10Command {
            wrapper, 
            block_address, 
            _transfer_bytes : transfer_bytes, 
            _block_size : block_size,
            transfer_blocks, 
        })
    }
}

impl BufferPushable for Read10Command {
    fn push_to_buffer<B : Buffer>(&self, buffer: &mut B) -> Result<usize, AumsError> {
        let mut rval = self.wrapper.push_to_buffer(buffer)?;
        rval += Read10Command::opcode().to_be().push_to_buffer(buffer)?;
        rval += buffer.push_byte(0)?;
        rval += self.block_address.to_be().push_to_buffer(buffer)?;
        rval += self.transfer_blocks.to_be().push_to_buffer(buffer)?;
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

    fn wrapper(&self) -> CommmandBlockWrapper {
        self.wrapper
    }
}