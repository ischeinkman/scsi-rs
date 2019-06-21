use error::{ErrorCause, ScsiError};
use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{ BufferPullable, BufferPushable};

use byteorder::{ByteOrder, BE};

/// Command to read capacity information about the block device. 
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
    fn push_to_buffer<B : AsMut<[u8]>>(&self, mut buffer: B) -> Result<usize, ScsiError> {
        let rval = self.wrapper().push_to_buffer(buffer.as_mut())?;
        buffer.as_mut()[rval] = ReadCapacityCommand::opcode();
        Ok(rval + 1)
    }
}

impl BufferPullable for ReadCapacityCommand {
    fn pull_from_buffer<B : AsRef<[u8]>>(buffer: B) -> Result<Self, ScsiError> {
        let wrapper: CommandBlockWrapper = CommandBlockWrapper::pull_from_buffer(buffer.as_ref())?;
        let opcode = buffer.as_ref()[15];
        if wrapper.data_transfer_length != 0x8
            || wrapper.direction != Direction::IN
            || wrapper.cb_length != ReadCapacityCommand::length()
            || opcode != ReadCapacityCommand::opcode()
        {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        Ok(ReadCapacityCommand::new())
    }
}

/// Response from an executed ReadCapacityCommand with capacity information.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ReadCapacityResponse {
    pub logical_block_address: u32,
    pub block_length: u32,
}

impl BufferPullable for ReadCapacityResponse {
    fn pull_from_buffer<B : AsRef<[u8]>>(buffer:  B) -> Result<ReadCapacityResponse, ScsiError> {
        let buffer = buffer.as_ref();
        let lba_bytes = BE::read_u32(buffer);
        let len_bytes = BE::read_u32(&buffer[4..]);
        Ok(ReadCapacityResponse {
            logical_block_address: lba_bytes,
            block_length: len_bytes,
        })
    }
}

impl BufferPushable for ReadCapacityResponse {
    fn push_to_buffer<B : AsMut<[u8]>>(&self, mut buffer: B) -> Result<usize, ScsiError> {
        let buffer = buffer.as_mut();
        BE::write_u32(&mut buffer[0..], self.logical_block_address);
        BE::write_u32(&mut buffer[4..], self.block_length);
        Ok(8)
    }
}

#[cfg(test)]
mod tests {
    use super::{ReadCapacityCommand, ReadCapacityResponse};
        use crate::{BufferPullable, BufferPushable};

    #[test]
    pub fn test_readcapacitycommand() {
        let expected: [u8; 31] = [
            0x55, 0x53, 0x42, 0x43, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x80, 0x00,
            0x10, 0x25, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];
        let mut buff = [0 ; 32];
        let read_command = ReadCapacityCommand::new();
        let pushed = read_command.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 16);
        assert_eq!(&buff[0..expected.len() as usize], &expected);

        let pulled = ReadCapacityCommand::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, read_command);
    }
    #[test]
    pub fn test_readcapacityresponse() {
        let expected: [u8; 16] = [
            0xab, 0xcd, 0xef, 0x12, 0x23, 0x45, 0x67, 0x89, 0xAA, 0xBB, 0xCC, 0xDD, 0x00, 0x00,
            0x00, 0x00,
        ];
        let mut buff = [0 ; 32];
        let read_response = ReadCapacityResponse {
            logical_block_address: 0xabcd_ef12,
            block_length: 0x2345_6789,
        };
        let pushed = read_response.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 8);
        assert_eq!(
            &buff[0..pushed as usize],
            &expected[0..pushed as usize]
        );

        let pulled = ReadCapacityResponse::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, read_response);
    }
}
