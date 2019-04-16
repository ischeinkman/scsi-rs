use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{Buffer, BufferPushable, BufferPullable};
use error::{ScsiError, ErrorCause};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct TestUnitReady {}

impl TestUnitReady {
    pub fn new() -> TestUnitReady {
        TestUnitReady {}
    }
}

impl Command for TestUnitReady {
    fn opcode() -> u8 {
        0x0
    }
    fn length() -> u8 {
        0x6
    }
    fn wrapper(&self) -> CommandBlockWrapper {
        CommandBlockWrapper::new(0, Direction::NONE, 0, 0x6)
    }
}

impl BufferPushable for TestUnitReady {
    fn push_to_buffer<B: Buffer>(&self, buffer: &mut B) -> Result<usize, ScsiError> {
        let mut rval = self.wrapper().push_to_buffer(buffer)?;
        rval += TestUnitReady::opcode().push_to_buffer(buffer)?;
        Ok(rval)
    }
}

impl BufferPullable for TestUnitReady {
    fn pull_from_buffer<B : Buffer>(buffer: &mut B) -> Result<Self, ScsiError> {
        let wrapper : CommandBlockWrapper = buffer.pull()?;
        if wrapper.data_transfer_length != 0 || wrapper.direction != Direction::NONE || wrapper.cb_length != TestUnitReady::length() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError))
        }
        let opcode = buffer.pull_byte()?;
        if opcode != TestUnitReady::opcode() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError))
        }
        Ok(TestUnitReady::new())
    }
}