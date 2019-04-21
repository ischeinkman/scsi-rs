use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{ BufferPullable, BufferPushable};
use error::{ScsiError, ErrorCause};
use byteorder::{BE, ByteOrder};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct InquiryCommand {
    pub allocation_length: u8,
}

impl InquiryCommand {
    pub fn new(allocation_length: u8) -> InquiryCommand {
        InquiryCommand { allocation_length }
    }
}

impl BufferPushable for InquiryCommand {
    fn push_to_buffer<B : AsMut<[u8]>>(&self, mut buffer: B) -> Result<usize, ScsiError> {
        let mut buffer = buffer.as_mut();
        let cur_idx = self.wrapper().push_to_buffer(&mut buffer)?;
        buffer[cur_idx] = InquiryCommand::opcode();
        buffer[cur_idx + 1] = 0;
        buffer[cur_idx + 2] = 0;
        buffer[cur_idx + 3] = 0;
        buffer[cur_idx + 4] = self.allocation_length;
        Ok(cur_idx + 5)
    }
}

impl BufferPullable for InquiryCommand {
    fn pull_from_buffer<B : AsRef<[u8]>>(buffer: B) -> Result<Self, ScsiError> {
        let buffer = buffer.as_ref();
        let header = CommandBlockWrapper::pull_from_buffer(buffer)?;
        let opcode = buffer[15];
        if opcode != InquiryCommand::opcode() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        let allocation_length_with_padding = BE::read_u32(&buffer[16..]);
        
        let allocation_length = allocation_length_with_padding as u8;

        if !header.data_transfer_length == allocation_length_with_padding || header.direction != Direction::IN || header.cb_length != InquiryCommand::length() {
            Err(ScsiError::from_cause(ErrorCause::ParseError))
        }
        else {
            Ok(InquiryCommand::new(allocation_length))

        }
    }
}

impl Command for InquiryCommand {
    fn wrapper(&self) -> CommandBlockWrapper {
        CommandBlockWrapper::new(
            u32::from(self.allocation_length),
            Direction::IN,
            0,
            InquiryCommand::length(),
        )
    }

    fn opcode() -> u8 {
        0x12
    }

    fn length() -> u8 {
        0x6
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct InquiryResponse {
    pub device_qualifier: u8,
    pub device_type: u8,
    _removable_flags: u8,
    _spc_version: u8,
    _response_format: u8,
}

impl BufferPullable for InquiryResponse {
    fn pull_from_buffer<B : AsRef<[u8]>>(buffer: B) -> Result<InquiryResponse, ScsiError> {
        let buffer = buffer.as_ref();
        let bt =buffer[0];
        let device_qualifier = bt & 0xe0;
        let device_type = bt & 0x1f;
        let _removable_flags = buffer[1];
        let _spc_version = buffer[2];
        let _response_format = buffer[3];
        Ok(InquiryResponse {
            device_qualifier,
            device_type,
            _removable_flags,
            _spc_version,
            _response_format,
        })
    }
}

impl BufferPushable for InquiryResponse {
    fn push_to_buffer<B : AsMut<[u8]>>(&self, mut buffer: B) -> Result<usize, ScsiError> {
        let buffer = buffer.as_mut();
        let bt = self.device_qualifier | self.device_type;
        buffer[0] = bt;
        buffer[1] = self._removable_flags;
        buffer[2] = self._spc_version;
        buffer[3] = self._response_format;
        Ok(4)
    }
}
#[cfg(test)]
mod tests {
    use crate::{BufferPullable, BufferPushable};
    use super::{InquiryCommand, InquiryResponse};

    #[test]
    pub fn test_inquirycommand() {
        let expected : [u8 ; 31] = [
            0x55, 0x53, 0x42, 0x43, 
            0x00, 0x00, 0x00, 0x00, 
            0x05, 0x00, 0x00, 0x00,
            0x80, 
            0x00, 
            0x06, 

            0x12, 
            0x00, 0x00, 0x00, 
            0x05, 
            0x00, 0x00, 
            0x00, 0x00,

            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ];
        let mut buff = [0 ; 32];
        let inquiry_command = InquiryCommand::new(0x5);
        let pushed = inquiry_command.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 20);
        assert_eq!(&buff[0 .. pushed], &expected[0 ..pushed]);

        let pulled = InquiryCommand::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, inquiry_command);
    }
    #[test]
    pub fn test_inquiryresponse() {
        let expected : [u8 ; 31] = [
            0xab, 0x56, 0x78, 0x9a, 
            0x00, 0x00, 0x00, 0x00, 
            0x05, 0x00, 0x00, 0x00,
            0x80, 
            0x00, 
            0x06, 

            0x12, 
            0x00, 0x00, 0x00, 
            0x05, 
            0x00, 0x00, 
            0x00, 0x00,

            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ];
        let mut buff = [0 ; 32];
        let inquiry_command = InquiryResponse{
            device_qualifier: 0xa0,
            device_type: 0x0b,
            _removable_flags: 0x56,
            _spc_version: 0x78,
            _response_format: 0x9a,
            
        };
        let pushed = inquiry_command.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 4);
        assert_eq!(&buff[0 .. pushed], &expected[0 ..pushed]);

        let pulled = InquiryResponse::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, inquiry_command);
    }
}