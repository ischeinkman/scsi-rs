use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{Buffer, BufferPullable, BufferPushable};
use error::ScsiError;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct ReadCapacityCommand {}

impl ReadCapacityCommand {
    pub fn new() -> ReadCapacityCommand {
        ReadCapacityCommand {}
    }
}

impl Command for ReadCapacityCommand {
    fn opcode() -> u8 {
        0x25
    }
    fn length() -> u8 {
        0x10
    }
    fn wrapper(&self) -> CommandBlockWrapper {
        CommandBlockWrapper::new(0x8, Direction::IN, 0, ReadCapacityCommand::length())
    }
}

impl BufferPushable for ReadCapacityCommand {
    fn push_to_buffer<B: Buffer>(&self, buffer: &mut B) -> Result<usize, ScsiError> {
        let mut rval = self.wrapper().push_to_buffer(buffer)?;
        rval += buffer.push_byte(ReadCapacityCommand::opcode())?;
        Ok(rval)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ReadCapacityResponse {
    pub logical_block_address: u32,
    pub block_length: u32,
}

impl BufferPullable for ReadCapacityResponse {
    fn pull_from_buffer<B: Buffer>(buffer: &mut B) -> Result<ReadCapacityResponse, ScsiError> {
        let lba_bytes = buffer.pull_u32_be()?;
        let len_bytes = buffer.pull_u32_be()?;
        Ok(ReadCapacityResponse {
            logical_block_address: lba_bytes,
            block_length: len_bytes,
        })
    }
}
