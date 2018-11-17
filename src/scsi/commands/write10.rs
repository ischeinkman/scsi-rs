use scsi::commands::{Direction, CommmandBlockWrapper, Command};
use traits::{Buffer, BufferPushable};
use crate::{AumsError};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Write10Command {
    block_address : u32, 
    transfer_bytes : u32, 
    block_size : u32, 
    transfer_blocks : u16,
}

impl Command for Write10Command {
    fn opcode() -> u8 {
        0x2a
    }
    fn length() -> u8 {
        10
    }
    fn wrapper(&self) -> CommmandBlockWrapper {
        CommmandBlockWrapper::new(0, Direction::OUT, 0, Write10Command::length())
    }
}

impl BufferPushable for Write10Command {
    fn push_to_buffer<B : Buffer>(&self, buffer: &mut B) -> Result<usize, AumsError> {
        let mut rval = self.wrapper().push_to_buffer(buffer)?;
        rval += Write10Command::opcode().swap_bytes().push_to_buffer(buffer)?;
        rval += buffer.push_byte(0)?;
        rval += self.block_address.swap_bytes().push_to_buffer(buffer)?;
        rval += buffer.push_byte(0)?;
        rval += self.transfer_blocks.swap_bytes().push_to_buffer(buffer)?;
        Ok(rval)
    }
}