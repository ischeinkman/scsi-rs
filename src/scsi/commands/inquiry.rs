
use scsi::commands::{CommmandBlockWrapper, Command, Direction};
use traits::{Buffer, BufferPullable, BufferPushable};
use crate::{AumsError, ErrorCause};


pub struct InquiryCommand {
    wrapper : CommmandBlockWrapper,
    allocation_length : u8,
}

impl InquiryCommand {
    pub fn new(allocation_length : u8) -> InquiryCommand {
        let wrapper = CommmandBlockWrapper::new(allocation_length as u32, Direction::IN, 0, InquiryCommand::length());
        InquiryCommand {
            wrapper, 
            allocation_length
        }
    }
}

impl BufferPushable for InquiryCommand {
    fn push_to_buffer<B : Buffer>(&self, buffer: &mut B) -> Result<usize, AumsError> {
        let mut rval = 0;
        rval += self.wrapper.push_to_buffer(buffer)?;
        rval += InquiryCommand::opcode().push_to_buffer(buffer)?;
        rval += buffer.push_byte(0)?;
        rval += buffer.push_byte(0)?;
        rval += buffer.push_byte(0)?;
        rval += buffer.push_byte(self.allocation_length)?;
        Ok(rval)
    }
}

impl Command for InquiryCommand {
    fn wrapper(&self) -> CommmandBlockWrapper {
        self.wrapper
    }

    fn opcode() -> u8 {
        0x12
    }

    fn length() -> u8 {
        0x6
    }
}


pub struct InquiryResponse {
    device_qualifier : u8, 
    device_type : u8, 
    is_removeable : bool, 
    spc_version : u8, 
    response_format : u8, 
}

impl BufferPullable for InquiryResponse {
    fn pull_from_buffer<B : Buffer>(buffer: &mut B) -> Result<InquiryResponse, AumsError> {
        let bt = buffer.pull_byte()?;
        let device_qualifier = bt & 0xe0;
        let device_type = bt & 0x1f; 
        let is_removable_raw = buffer.pull_byte()?;
        let is_removeable = is_removable_raw == 0x80;
        let spc_version = buffer.pull_byte()?;
        let response_format = buffer.pull_byte()? & 0x7;
        Ok(InquiryResponse {
            device_qualifier, 
            device_type, 
            is_removeable, 
            spc_version, 
            response_format
        })
    }
}
