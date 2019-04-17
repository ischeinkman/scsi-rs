use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{Buffer, BufferPullable, BufferPushable};
use error::{ScsiError, ErrorCause};

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
    fn push_to_buffer<B: Buffer>(&self, buffer: &mut B) -> Result<usize, ScsiError> {
        let mut rval = 0;
        rval += self.wrapper().push_to_buffer(buffer)?;
        rval += buffer.push_byte(InquiryCommand::opcode())?;
        rval += buffer.push_byte(0)?;
        rval += buffer.push_byte(0)?;
        rval += buffer.push_byte(0)?;
        rval += buffer.push_byte(self.allocation_length)?;
        Ok(rval)
    }
}

impl BufferPullable for InquiryCommand {
    fn pull_from_buffer<B : Buffer>(buffer: &mut B) -> Result<Self, ScsiError> {
        let header = CommandBlockWrapper::pull_from_buffer(buffer)?;
        let opcode = buffer.pull_byte()?;
        if opcode != InquiryCommand::opcode() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        let allocation_length_with_padding = buffer.pull_u32_be()?;
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
    fn pull_from_buffer<B: Buffer>(buffer: &mut B) -> Result<InquiryResponse, ScsiError> {
        let bt = buffer.pull_byte()?;
        let device_qualifier = bt & 0xe0;
        let device_type = bt & 0x1f;
        let _removable_flags = buffer.pull_byte()?;
        let _spc_version = buffer.pull_byte()?;
        let _response_format = buffer.pull_byte()?;
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
    fn push_to_buffer<B : Buffer>(&self, buffer: &mut B) -> Result<usize, ScsiError> {
        let mut rval = 0;
        let bt = self.device_qualifier | self.device_type;
        rval += buffer.push_byte(bt)?;
        rval += buffer.push_byte(self._removable_flags)?;
        rval += buffer.push_byte(self._spc_version)?;
        rval += buffer.push_byte(self._response_format)?;
        Ok(rval)
    }
}
#[cfg(test)]
mod tests {
    use crate::traits::test::VecNewtype;
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
        let mut buff = VecNewtype::new();
        let inquiry_command = InquiryCommand::new(0x5);
        let pushed = inquiry_command.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 20);
        assert_eq!(&buff.inner[0 .. pushed], &expected[0 ..pushed]);

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
        let mut buff = VecNewtype::new();
        let inquiry_command = InquiryResponse{
            device_qualifier: 0xa0,
            device_type: 0x0b,
            _removable_flags: 0x56,
            _spc_version: 0x78,
            _response_format: 0x9a,
            
        };
        let pushed = inquiry_command.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 20);
        assert_eq!(&buff.inner[0 .. pushed], &expected[0 ..pushed]);

        let pulled = InquiryResponse::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, inquiry_command);
    }
}