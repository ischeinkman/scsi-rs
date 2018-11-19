use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{Buffer, BufferPushable};
use error::ScsiError;

pub struct RequestSenseCommand {
    allocation_length: u8,
}

impl Command for RequestSenseCommand {
    fn opcode() -> u8 {
        0x3
    }
    fn length() -> u8 {
        0x6
    }
    fn wrapper(&self) -> CommandBlockWrapper {
        CommandBlockWrapper::new(0, Direction::NONE, 0, RequestSenseCommand::length())
    }
}

impl BufferPushable for RequestSenseCommand {
    fn push_to_buffer<B: Buffer>(&self, buffer: &mut B) -> Result<usize, ScsiError> {
        let mut rval = self.wrapper().push_to_buffer(buffer)?;
        rval += RequestSenseCommand::opcode().push_to_buffer(buffer)?;
        rval += buffer.push_byte(0)?;
        rval += buffer.push_byte(0)?;
        rval += buffer.push_byte(0)?;
        rval += self.allocation_length.push_to_buffer(buffer)?;
        Ok(rval)
    }
}
