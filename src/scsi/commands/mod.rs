mod inquiry;
pub use self::inquiry::*;
mod read10;
pub use self::read10::*;
mod readcapacity;
pub use self::readcapacity::*;
mod requestsense;
pub use self::requestsense::*;
mod testunit;
pub use self::testunit::*;
mod write10;
pub use self::write10::*;

use traits::{Buffer, BufferPullable, BufferPushable};
use error::{ErrorCause, ScsiError};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Direction {
    IN,
    OUT,
    NONE,
}

/// A struct that prefaces all commands in the SCSI protocol. 
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct CommandBlockWrapper {
    //TODO: What are these fields?
    pub tag: u32,
    pub data_transfer_length: u32,
    pub flags: u8,
    pub lun: u8,
    pub cb_length: u8,
    pub direction: Direction,
}

impl CommandBlockWrapper {
    /// A magic number that should preface the Command Block Wrapper on the buffer.
    pub const D_CBW_SIGNATURE: u32 = 0x4342_5355;

    /// Constructs a new CommandBlockWrapper.
    ///
    /// Currently, `tag` is by default set to 0 and the only flag set is the
    /// direction flag, which is either `0x80` when `direction` is `Direction::IN`
    /// and `0` otherwise.
    pub fn new(
        data_transfer_length: u32,
        direction: Direction,
        lun: u8,
        cb_length: u8,
    ) -> CommandBlockWrapper {
        let direction_flags: u8 = match direction {
            Direction::IN => 0x80,
            _ => 0,
        };
        CommandBlockWrapper {
            tag: 0, //TODO: Is this the correct default?
            data_transfer_length,
            flags: direction_flags,
            lun,
            cb_length,
            direction,
        }
    }
}

impl BufferPushable for CommandBlockWrapper {
    fn push_to_buffer<B: Buffer>(&self, buffer: &mut B) -> Result<usize, ScsiError> {
        let mut rval = 0;
        rval += buffer.push_u32_le(CommandBlockWrapper::D_CBW_SIGNATURE)?;
        rval += buffer.push_u32_le(self.tag)?;
        rval += buffer.push_u32_le(self.data_transfer_length)?;

        rval += buffer.push_byte(self.flags)?;
        rval += buffer.push_byte(self.lun)?;
        rval += buffer.push_byte(self.cb_length)?;
        Ok(rval)
    }
}

/// A trait that all SCSI commands must implement.
pub trait Command: BufferPushable {
    /// Returns the command block that prefaces this command struct.
    fn wrapper(&self) -> CommandBlockWrapper;

    /// Returns the specific opcode of this command.
    fn opcode() -> u8;

    /// Returns the length of the command call, usually either 6, 10, or 16.
    fn length() -> u8;
}

/// This struct prefaces all responses from the SCSI device when a command
/// requires a response. 
pub struct CommandStatusWrapper {
    pub tag: u32,
    pub data_residue: u32,
    pub status: u8,
}

impl CommandStatusWrapper {
    /// The value of the `status` field if the initating command succeeded.
    pub const COMMAND_PASSED: u32 = 0;

    /// The value of the `status` field if the initating command failed.
    pub const COMMAND_FAILED: u32 = 1;
    /// The value of the `status` field if the initating command encountered a
    /// phace error.
    pub const PHASE_ERROR: u32 = 2;
    /// The size of the Command Status Wrapper, including magic number, in bytes.
    pub const SIZE: u32 = 13;

    /// A magic number that should preface the Command Status Wrapper on the buffer.
    pub const D_CSW_SIGNATURE: u32 = 0x5342_5355;
}

impl BufferPullable for CommandStatusWrapper {
    fn pull_from_buffer<B: Buffer>(buffer: &mut B) -> Result<Self, ScsiError> {
        let signature = buffer.pull_u32_le()?;
        if signature != CommandStatusWrapper::D_CSW_SIGNATURE {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        let tag = buffer.pull_u32_le()?;
        let data_residue = buffer.pull_u32_le()?;
        let status = buffer.pull_byte()?;

        Ok(CommandStatusWrapper {
            tag,
            data_residue,
            status,
        })
    }
}
