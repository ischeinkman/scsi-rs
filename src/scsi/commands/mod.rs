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

impl From<u8> for Direction {
    fn from(flags : u8) -> Direction {
        match flags & 0x80 {
            0 => Direction::OUT,
            _ => Direction::IN,
        }
    }
}

impl From<Direction> for u8 {
    fn from(dir : Direction) -> u8 {
        match dir {
            Direction::IN => 0x80,
            _ => 0x0,
        }
    }
}

/// A struct that prefaces all commands in the SCSI protocol. 
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct CommandBlockWrapper {

    /// The identifier of the command this CBW is wrapping
    pub tag: u32,

    /// How much non-command data needs to be transfered for this command 
    /// (eg data being read for a Read10, data being written for a Write10, etc)
    pub data_transfer_length: u32,

    /// General flags about the command to be executed; currently only supports
    /// the most significant bit, which is 1 when the command requires the device to
    /// send data back to the host and 0 otherwise.
    pub flags: u8,

    /// TODO: What is this?
    pub lun: u8,

    /// The length of the command parameters to be executed, not counting external data to be transfered. 
    pub cb_length: u8,

    /// The direction data will be flowing in, either IN for device -> host, OUT for host -> device, 
    /// and NONE if the command has no associated data transfer.
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

impl BufferPullable for CommandBlockWrapper {
    fn pull_from_buffer<B : Buffer>(buffer: &mut B) -> Result<Self, ScsiError> {
        let magic = buffer.pull_u32_le()?;
        if magic != CommandBlockWrapper::D_CBW_SIGNATURE {
            return Err(ScsiError::from_cause(ErrorCause::FlagError{flags : magic}));
        }

        let tag = buffer.pull_u32_le()?;
        let data_transfer_length = buffer.pull_u32_le()?;

        let flags = buffer.pull_byte()?;
        let lun = buffer.pull_byte()?;
        let cb_length = buffer.pull_byte()?;

        Ok(CommandBlockWrapper {
            tag,
            data_transfer_length,
            flags,
            lun,
            cb_length,
            direction : flags.into(),
        })
    }
}

/// A trait that all SCSI commands must implement.
pub trait Command: BufferPushable + BufferPullable {
    /// Returns the command block that prefaces this command struct.
    fn wrapper(&self) -> CommandBlockWrapper;

    /// Returns the specific opcode of this command.
    fn opcode() -> u8;

    /// Returns the length of the command call, usually either 6, 10, or 16.
    fn length() -> u8;
}

/// This struct prefaces all responses from the SCSI device when a command
/// requires a response. 
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct CommandStatusWrapper {
    pub tag: u32,
    pub data_residue: u32,
    pub status: u8,
}

impl CommandStatusWrapper {
    /// The value of the `status` field if the initating command succeeded.
    pub const COMMAND_PASSED: u8 = 0;

    /// The value of the `status` field if the initating command failed.
    pub const COMMAND_FAILED: u8 = 1;
    /// The value of the `status` field if the initating command encountered a
    /// phace error.
    pub const PHASE_ERROR: u8 = 2;
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

impl BufferPushable for CommandStatusWrapper {
    fn push_to_buffer<B : Buffer>(&self, buffer: &mut B) -> Result<usize, ScsiError> {
        let mut rval = 0;
        rval += buffer.push_u32_le(CommandStatusWrapper::D_CSW_SIGNATURE)?;
        rval += buffer.push_u32_le(self.tag)?;
        rval += buffer.push_u32_le(self.data_residue)?;
        rval += buffer.push_byte(self.status)?;
        Ok(rval)
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandBlockWrapper, CommandStatusWrapper, Direction};
    use crate::traits::test::VecNewtype;
    use crate::{BufferPullable, BufferPushable};

    #[test]
    pub fn test_cbw() {
        let expected: [u8; 31] = [
            0x55, 0x53, 0x42, 0x43, 0x00, 0x00, 0x00, 0x00, 0x12, 0xef, 0xcd, 0xab, 0x80, 0x34,
            0x56, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];
        let mut buff = VecNewtype::new();
        let cbw = CommandBlockWrapper::new(0xabcdef12, Direction::IN, 0x34, 0x56);
        let pushed = cbw.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 15);
        assert_eq!(&buff.inner[0..pushed], &expected[0 .. pushed]);

        let pulled = CommandBlockWrapper::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, cbw);
    }
    #[test]
    pub fn test_csw() {
        let expected: [u8; 31] = [
            0x55, 0x53, 0x42, 0x53, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90, 0x80, 0x34,
            0x56, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];
        let mut buff = VecNewtype::new();
        let csw = CommandStatusWrapper{tag : 0x12efcdab, data_residue : 0x90785634, status : 0x80};
        let pushed = csw.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 13);
        assert_eq!(&buff.inner[0..pushed], &expected[0 .. pushed]);

        let pulled = CommandStatusWrapper::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, csw);
    }
}
