use error::{ErrorCause, ScsiError};
use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{Buffer, BufferPullable, BufferPushable};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct RequestSenseCommand {
    allocation_length: u8,
}

impl RequestSenseCommand {
    pub fn new(allocation_length: u8) -> Self {
        RequestSenseCommand { allocation_length }
    }
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

impl BufferPullable for RequestSenseCommand {
    fn pull_from_buffer<B: Buffer>(buffer: &mut B) -> Result<Self, ScsiError> {
        let wrapper: CommandBlockWrapper = buffer.pull()?;
        if wrapper.data_transfer_length != 0
            || !(wrapper.direction == Direction::NONE || wrapper.direction == Direction::OUT)
            || wrapper.cb_length != RequestSenseCommand::length()
        {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        let opcode_with_padding = buffer.pull_u32_le()?;
        if opcode_with_padding != RequestSenseCommand::opcode().into() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        let allocation_length = buffer.pull_byte()?;
        Ok(RequestSenseCommand::new(allocation_length))
    }
}

#[cfg(test)]
mod tests {
    use super::RequestSenseCommand;
    use crate::traits::test::VecNewtype;
    use crate::{BufferPullable, BufferPushable};

    #[test]
    pub fn test_requestsense() {
        let expected: [u8; 31] = [
            0x55, 0x53, 0x42, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0, 0x00,
            0x06, 0x03, 0x00, 0x00, 0x00, 0x0A, 0x0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];
        let mut buff = VecNewtype::new();
        let tur_command = RequestSenseCommand::new(0xA);
        let pushed = tur_command.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 20);
        assert_eq!(&buff.inner[0..pushed], &expected[0 .. pushed]);

        let pulled = RequestSenseCommand::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, tur_command);
    }
}
