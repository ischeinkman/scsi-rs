

pub mod inquiry;
pub mod read10;
pub mod readcapacity;
pub mod write10; 
pub mod testunit;
pub mod requestsense; 
use traits::{Buffer, BufferPushable, BufferPullable};
use crate::AumsError;
use crate::ErrorCause;


#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Direction {
    IN, 
    OUT, 
    NONE, 
}
pub const D_CBW_SIGNATURE : u32 = 0x43425355; 

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct CommmandBlockWrapper {
    //TODO: What are these fields?
    tag : u32, 
    data_transfer_length : u32, 
    flags : u8, 
    lun : u8, 
    cb_length : u8, 
    direction : Direction,
}

impl CommmandBlockWrapper {
    pub fn new(transfer_length : u32, direction : Direction, lun : u8, cb_length : u8) -> CommmandBlockWrapper {
        let direction_flags : u8 = match direction {
            Direction::IN => 0x80, 
            _ => 0,
        };
        CommmandBlockWrapper {
            tag : 0, //TODO: Is this the correct default?
            data_transfer_length : transfer_length, 
            flags : direction_flags,
            lun, 
            cb_length,
            direction, 
        }
    }
}

impl BufferPushable for CommmandBlockWrapper {
    fn push_to_buffer<B : Buffer>(&self, buffer: &mut B) -> Result<usize, AumsError> {
        let mut rval = 0;
        rval += D_CBW_SIGNATURE.push_to_buffer(buffer)?;
        rval += self.tag.push_to_buffer(buffer)?;
        rval += self.data_transfer_length.push_to_buffer(buffer)?;
        rval += self.flags.push_to_buffer(buffer)?;
        rval += self.lun.push_to_buffer(buffer)?;
        rval += self.cb_length.push_to_buffer(buffer)?;
        Ok(rval)
    }
}

pub trait Command : BufferPushable {
    fn wrapper(&self) -> CommmandBlockWrapper;
    fn opcode() -> u8; 
    fn length() -> u8; 
}
const COMMAND_PASSED : u32 = 0; 
const COMMAND_FAILED : u32 = 1; 
const PHASE_ERROR : u32 = 2; 
const SIZE : u32 = 13; 
const D_CSW_SIGNATURE : u32 = 0x53425355;

pub struct CommandStatusWrapper {
    tag : u32, 
    data_residue : u32, 
    status : u8,
}

impl BufferPullable for CommandStatusWrapper {
    fn pull_from_buffer<B : Buffer>(buffer: &mut B) -> Result<Self, AumsError> {
        let signature = u32::pull_from_buffer(buffer)?;
        if signature != D_CSW_SIGNATURE {
            return Err(AumsError::from_cause(ErrorCause::ParseError));
        }
        let tag = u32::pull_from_buffer(buffer)?;
        let data_residue = u32::pull_from_buffer(buffer)?;
        let status = u8::pull_from_buffer(buffer)?;

        Ok(CommandStatusWrapper{
            tag, 
            data_residue, 
            status
        })
    }
}