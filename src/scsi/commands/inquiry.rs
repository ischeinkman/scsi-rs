use byteorder::{ByteOrder, BE};
use error::{ErrorCause, ScsiError};
use scsi::commands::{Command, CommandBlockWrapper, Direction};
use traits::{BufferPullable, BufferPushable};

/// A command to get information about an SCSI device.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct InquiryCommand {
    /// The size of the response that should be returned.
    ///
    /// Many devices only support an `allocation_length` of 36; other values
    /// should be used with care.
    pub allocation_length: u8,
}

impl InquiryCommand {
    /// Constructs a new `InquiryCommand` with the given value for
    /// `allocation_length`.
    pub fn new(allocation_length: u8) -> InquiryCommand {
        InquiryCommand { allocation_length }
    }
}

impl BufferPushable for InquiryCommand {
    fn push_to_buffer<B: AsMut<[u8]>>(&self, mut buffer: B) -> Result<usize, ScsiError> {
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
    fn pull_from_buffer<B: AsRef<[u8]>>(buffer: B) -> Result<Self, ScsiError> {
        let buffer = buffer.as_ref();
        let header = CommandBlockWrapper::pull_from_buffer(buffer)?;
        let opcode = buffer[15];
        if opcode != InquiryCommand::opcode() {
            return Err(ScsiError::from_cause(ErrorCause::ParseError));
        }
        let allocation_length_with_padding = BE::read_u32(&buffer[16..]);

        let allocation_length = allocation_length_with_padding as u8;

        if !header.data_transfer_length == allocation_length_with_padding
            || header.direction != Direction::IN
            || header.cb_length != InquiryCommand::length()
        {
            Err(ScsiError::from_cause(ErrorCause::ParseError))
        } else {
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

/// The data sent in response to an `InquiryCommand`.
///
/// Currently does not include all data that the device responds with; finishing
/// this is a TODO item.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
pub struct InquiryResponse {
    /// 3 bit flag set to determine the SCSI device's current accessibility. For
    /// most common devices, this will be 0 to indicate that the device is accessible
    /// for running commands.
    ///
    /// The flag bits are as follows:
    ///
    /// * If the least significant bit, `0x20`, is set, then the "SCSI task router"
    /// in use is currently unable to access the logical unit specified via the LUN
    /// field in the CBW.
    ///
    /// * If the next bit, `0x40`, is set, then specified LUN cannot be accessed
    /// by the current "SCSI task router". Note that this flag being set implies
    /// that the previous bit must also be set, and that the returned value of
    /// `device_type` is `0x1F` to indicate a value of `Unknown`.
    ///
    /// * If the most significant bit, `0x80`, is set, then this flat set must be
    /// interpretted through the device vendor's documentation instead of the information
    /// given here; the previous 2 bullets no longer apply.
    pub device_qualifier: u8,

    /// The type of SCSI device this is. Usually 0 to indicate a
    /// randomly accessable block storage device.
    ///
    /// For valid values, see the [SCSI Peripheral Device Type](https://en.wikipedia.org/wiki/SCSI_Peripheral_Device_Type)
    /// list.
    pub device_type: u8,

    /// A flag set whose most significant bit corresponds to whether or not the
    /// device is removable.
    ///
    /// Currently the next bit corresponds to whether or not the logical unit
    /// is part of a "logical unit conglomerate", while the rest are reserved for
    /// future use.
    pub removable_flags: u8,

    /// Indicates what version of the SCSI command set this device conforms to;
    /// at time of writing, a value of 7 corresponds to adherence to the latest
    /// version of the command specifications, SPC-5.
    ///
    /// A value of 0 indicates that the device does not claim to match *any*
    /// SCSI specification version; proceed with caution in that case.
    pub spc_version: u8,

    /// What format the response will be in.
    ///
    /// Currently, the only valid value is 2.
    pub response_format: u8,
}

impl BufferPullable for InquiryResponse {
    fn pull_from_buffer<B: AsRef<[u8]>>(buffer: B) -> Result<InquiryResponse, ScsiError> {
        let buffer = buffer.as_ref();
        let bt = buffer[0];
        let device_qualifier = bt & 0xe0;
        let device_type = bt & 0x1f;
        let removable_flags = buffer[1];
        let spc_version = buffer[2];
        let response_format = buffer[3];
        Ok(InquiryResponse {
            device_qualifier,
            device_type,
            removable_flags,
            spc_version,
            response_format,
        })
    }
}

impl BufferPushable for InquiryResponse {
    fn push_to_buffer<B: AsMut<[u8]>>(&self, mut buffer: B) -> Result<usize, ScsiError> {
        let buffer = buffer.as_mut();
        let bt = self.device_qualifier | self.device_type;
        buffer[0] = bt;
        buffer[1] = self.removable_flags;
        buffer[2] = self.spc_version;
        buffer[3] = self.response_format;
        Ok(4)
    }
}
#[cfg(test)]
mod tests {
    use super::{InquiryCommand, InquiryResponse};
    use crate::{BufferPullable, BufferPushable};

    #[test]
    pub fn test_inquirycommand() {
        let expected: [u8; 31] = [
            0x55, 0x53, 0x42, 0x43, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x80, 0x00,
            0x06, 0x12, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];
        let mut buff = [0; 32];
        let inquiry_command = InquiryCommand::new(0x5);
        let pushed = inquiry_command.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 20);
        assert_eq!(&buff[0..pushed], &expected[0..pushed]);

        let pulled = InquiryCommand::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, inquiry_command);
    }
    #[test]
    pub fn test_inquiryresponse() {
        let expected: [u8; 31] = [
            0xab, 0x56, 0x78, 0x9a, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x80, 0x00,
            0x06, 0x12, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];
        let mut buff = [0; 32];
        let inquiry_command = InquiryResponse {
            device_qualifier: 0xa0,
            device_type: 0x0b,
            removable_flags: 0x56,
            spc_version: 0x78,
            response_format: 0x9a,
        };
        let pushed = inquiry_command.push_to_buffer(&mut buff).unwrap();
        assert_eq!(pushed, 4);
        assert_eq!(&buff[0..pushed], &expected[0..pushed]);

        let pulled = InquiryResponse::pull_from_buffer(&mut buff).unwrap();
        assert_eq!(pulled, inquiry_command);
    }
}
