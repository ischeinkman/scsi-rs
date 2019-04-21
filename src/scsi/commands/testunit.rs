use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{ BufferPushable, BufferPullable};
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
    fn push_to_buffer<B : AsMut<[u8]>>(&self, mut buffer: B) -> Result<usize, ScsiError> {
        let rval = self.wrapper().push_to_buffer(buffer.as_mut())?;
        buffer.as_mut()[rval] = TestUnitReady::opcode();
        Ok(rval + 1)
    }
}

impl BufferPullable for TestUnitReady {
    fn pull_from_buffer<B : AsRef<[u8]>>(buffer: B) -> Result<Self, ScsiError> {
        let wrapper : CommandBlockWrapper = CommandBlockWrapper::pull_from_buffer(buffer.as_ref())?;
        if wrapper.data_transfer_length != 0 || !(wrapper.direction == Direction::OUT || wrapper.direction == Direction::NONE)  || wrapper.cb_length != TestUnitReady::length() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError))
        }
        let buffer = &buffer.as_ref()[16 ..];
        let opcode = buffer[0];
        if opcode != TestUnitReady::opcode() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError))
        }
        Ok(TestUnitReady::new())
    }
}

#[cfg(test)]
mod tests {
    use super::TestUnitReady;
        use crate::{BufferPullable, BufferPushable};

    #[test]
    pub fn test_tur() {
        let expected: [u8; 31] = [
            0x55, 0x53, 0x42, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0, 0x00,
            0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];
        let mut buff = [0 ; 32];
        let tur_command = TestUnitReady::new();
        let pushed = tur_command.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 16);
        assert_eq!(&buff[0..pushed], &expected[0 .. pushed]);

        let pulled = TestUnitReady::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, tur_command);
    }
}
