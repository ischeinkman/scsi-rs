use error::{ErrorCause, ScsiError};
use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{ BufferPullable, BufferPushable};

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
    fn push_to_buffer<B : AsMut<[u8]>>(&self, mut buffer: B) -> Result<usize, ScsiError> {
        let rval = self.wrapper().push_to_buffer(buffer.as_mut())?;
        let buffer = &mut buffer.as_mut()[rval ..];
        buffer[0] = RequestSenseCommand::opcode();
        buffer[1] = 0;
        buffer[2] = 0;
        buffer[3] = 0;
        buffer[4] = self.allocation_length;
        Ok(rval + 5)
    }
}

impl BufferPullable for RequestSenseCommand {
    fn pull_from_buffer<B : AsRef<[u8]>>(buffer: B) -> Result<Self, ScsiError> {
        let wrapper: CommandBlockWrapper = CommandBlockWrapper::pull_from_buffer(buffer.as_ref())?;
        if wrapper.data_transfer_length != 0
            || !(wrapper.direction == Direction::NONE || wrapper.direction == Direction::OUT)
            || wrapper.cb_length != RequestSenseCommand::length()
        {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        let buffer = &buffer.as_ref()[15 ..];
        let opcode = buffer[0];
        if opcode != RequestSenseCommand::opcode() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        let allocation_length = buffer[4];
        Ok(RequestSenseCommand::new(allocation_length))
    }
}

#[cfg(test)]
mod tests {
    use super::RequestSenseCommand;
        use crate::{BufferPullable, BufferPushable};

    #[test]
    pub fn test_requestsense() {
        let expected: [u8; 31] = [
            0x55, 0x53, 0x42, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0, 0x00,
            0x06, 0x03, 0x00, 0x00, 0x00, 0x0A, 0x0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];
        let mut buff = [0 ; 32];
        let tur_command = RequestSenseCommand::new(0xA);
        let pushed = tur_command.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 20);
        assert_eq!(&buff[0..pushed], &expected[0 .. pushed]);

        let pulled = RequestSenseCommand::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, tur_command);
    }
}
