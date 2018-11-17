
use scsi::commands::{Direction, Command, CommmandBlockWrapper};
use traits::{Buffer, BufferPushable};
use crate::AumsError;


#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct TestUnitReady {}

impl TestUnitReady {
    pub fn new() -> TestUnitReady {
        TestUnitReady{}
    }
}

impl Command for TestUnitReady {
    fn opcode() -> u8 {
        0x0
    }
    fn length() -> u8 {
        0x6
    }
    fn wrapper(&self) -> CommmandBlockWrapper {
        CommmandBlockWrapper::new(0, Direction::NONE, 0, 0x6)
    }
} 

impl BufferPushable for TestUnitReady {
    fn push_to_buffer<B : Buffer>(&self, buffer: &mut B) -> Result<usize, AumsError> {
        let mut rval = self.wrapper().push_to_buffer(buffer)?;
        rval += TestUnitReady::opcode().push_to_buffer(buffer)?;
        Ok(rval)
    }
}