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
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
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