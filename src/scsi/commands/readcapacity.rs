use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{Buffer, BufferPullable, BufferPushable};
use error::{ScsiError, ErrorCause};

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

impl BufferPullable for ReadCapacityCommand {
    fn pull_from_buffer<B : Buffer>(buffer: &mut B) -> Result<Self, ScsiError> {
        let wrapper : CommandBlockWrapper = buffer.pull()?;
        let opcode = buffer.pull_byte()?;
        if wrapper.data_transfer_length != 0x8 || wrapper.direction != Direction::IN || wrapper.cb_length != ReadCapacityCommand::length() || opcode != ReadCapacityCommand::opcode() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        Ok(ReadCapacityCommand::new())
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

impl BufferPushable for ReadCapacityResponse {
    fn push_to_buffer<B : Buffer>(&self, buffer: &mut B) -> Result<usize, ScsiError> {
        let mut rval = 0;
        rval += buffer.push_u32_be(self.logical_block_address)?;
        rval += buffer.push_u32_be(self.block_length)?;
        Ok(rval)
    }
}