use error::{ErrorCause, ScsiError, UsbTransferDirection};
use scsi::commands::Read10Command;
use scsi::commands::TestUnitReady;
use scsi::commands::Write10Command;
use scsi::commands::{Command, CommandStatusWrapper, Direction};
use scsi::commands::{InquiryCommand, InquiryResponse};
use scsi::commands::{ReadCapacityCommand, ReadCapacityResponse};
use traits::{BufferPullable, CommunicationChannel};

/// A struct that provides a simple, block-device-like interface around an SCSI device.
/// This allows for reading and writing to the device at static offests, allowing for
/// easy interaction with any file system crate.
pub struct ScsiBlockDevice<CommType: CommunicationChannel> {
    pub comm_channel: CommType,
    block_size: u32,
    pub prev_csw: Option<CommandStatusWrapper>,
}

impl<CommType: CommunicationChannel> ScsiBlockDevice<CommType> {
    /// Constructs a new `ScsiBlockDevice`.
    /// # Parameters
    /// *  `comm_channel` is the communication channel to be used to send out commands and read the responses.  
    /// *  `scratch_buffer` is a buffer that will be used for the initialization commands and responses; it will not be used outside of this method itself.
    pub fn new(
        mut comm_channel: CommType,
        mut scratch_buffer: &mut [u8],
    ) -> Result<Self, ScsiError> {
        if scratch_buffer.len() < 31 {
            return Err(ScsiError::from_cause(ErrorCause::BufferTooSmallError {
                expected: 31,
                actual: scratch_buffer.len(),
            }));
        }
        let inquiry = InquiryCommand::new(scratch_buffer.len().min(36) as u8);
        let (_ir, _csw_ic) = transfer_in_command(&mut comm_channel, &inquiry, &mut scratch_buffer)?;
        let inquiry_resp = InquiryResponse::pull_from_buffer(&scratch_buffer)?;
        if inquiry_resp.device_qualifier != 0 || inquiry_resp.device_type != 0 {
            return Err(ScsiError::from_cause(ErrorCause::InvalidDeviceError));
        }

        let test_unit = TestUnitReady::new();
        let (_, _csw_tur) = transfer_out_command(&mut comm_channel, &test_unit, &scratch_buffer)?;

        let read_capacity = ReadCapacityCommand::new();
        let (_, mut csw_rcc) =
            transfer_in_command(&mut comm_channel, &read_capacity, &mut scratch_buffer)?;
        csw_rcc.tag = 2;
        let capacity_resp = ReadCapacityResponse::pull_from_buffer(&scratch_buffer)?;
        let block_size = capacity_resp.block_length;
        let rval = ScsiBlockDevice {
            comm_channel,
            block_size,
            prev_csw: Some(csw_rcc),
        };
        Ok(rval)
    }

    /// Reads bytes starting at `offset` into the provided `dest` buffer, returning
    /// the number of bytes read on success.
    pub fn read<B: AsMut<[u8]>>(&mut self, offset: u32, mut dest: B) -> Result<usize, ScsiError> {
        let buffer = dest.as_mut();
        let prev_tag = match &self.prev_csw {
            Some(ref c) => c.tag,
            None => 0,
        };
        self.prev_csw = None;
        if buffer.len() == 0 {
            return Ok(0);
        }
        if buffer.len() % self.block_size as usize != 0 {
            return Err(ScsiError::from_cause(
                ErrorCause::NonBlocksizeMultipleLengthError {
                    actual: buffer.len(),
                    block_size: self.block_size as usize,
                },
            ));
        }
        let read_command = Read10Command::new(offset, buffer.len() as u32, self.block_size)?;
        let (r, mut csw) = transfer_in_command(&mut self.comm_channel, &read_command, buffer)?;
        csw.tag = prev_tag + 1;
        self.prev_csw = Some(csw);
        Ok(r)
    }

    /// Writes bytes starting at `offset` from the provided buffer `src`, returning the
    /// number of bytes written on success.
    pub fn write<B: AsMut<[u8]>>(&mut self, offset: u32, mut src: B) -> Result<usize, ScsiError> {
        let buffer = src.as_mut();
        if buffer.len() == 0 {
            return Ok(0);
        }
        let prev_tag = match &self.prev_csw {
            Some(ref c) => c.tag,
            None => 0,
        };
        self.prev_csw = None;
        let to_transfer = buffer.len();
        if to_transfer % self.block_size as usize != 0 {
            return Err(ScsiError::from_cause(
                ErrorCause::NonBlocksizeMultipleLengthError {
                    actual: to_transfer,
                    block_size: self.block_size as usize,
                },
            ));
        }
        let write_command = Write10Command::new(offset, to_transfer as u32, self.block_size)?;
        let (w, mut csw) = transfer_out_command(&mut self.comm_channel, &write_command, buffer)?;
        csw.tag = prev_tag + 1;
        self.prev_csw = Some(csw);
        Ok(w)
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }
}

fn read_csw<C: CommunicationChannel>(
    comm_channel: &mut C,
) -> Result<CommandStatusWrapper, ScsiError> {
    let mut scratch_buffer = [0; CommandStatusWrapper::SIZE as usize];
    let read_count = comm_channel.in_transfer(&mut scratch_buffer)?;
    if read_count != CommandStatusWrapper::SIZE as usize {
        return Err(ScsiError::from_cause(ErrorCause::UsbTransferError {
            direction: UsbTransferDirection::In,
        }));
    }
    let retval = CommandStatusWrapper::pull_from_buffer(scratch_buffer)?;
    Ok(retval)
}

fn push_command<C: CommunicationChannel, Cmd: Command>(
    comm_channel: &mut C,
    command: &Cmd,
) -> Result<usize, ScsiError> {
    let scratch_buffer = [0; 31];
    // Push the command's bytes to the buffer
    let _serial_bytes = command.push_to_buffer(scratch_buffer)?;
    let pushed_bytes = comm_channel.out_transfer(scratch_buffer)?;
    if pushed_bytes != 31 {
        Err(ScsiError::from_cause(ErrorCause::UsbTransferError {
            direction: UsbTransferDirection::Out,
        }))
    } else {
        Ok(pushed_bytes)
    }
}

fn transfer_out_command<Usb: CommunicationChannel, C: Command, OutBuff: AsRef<[u8]>>(
    comm_channel: &mut Usb,
    command: &C,
    out_buffer: OutBuff,
) -> Result<(usize, CommandStatusWrapper), ScsiError> {
    let _command_bytes = push_command(comm_channel, command)?;

    let transfer_length = command.wrapper().data_transfer_length;
    let write = if transfer_length == 0 {
        0
    } else if command.wrapper().direction == Direction::IN {
        return Err(ScsiError::from_cause(ErrorCause::UnsupportedOperationError));
    } else {
        let mut written = comm_channel.out_transfer(out_buffer.as_ref())?;
        while written < transfer_length as usize {
            written += comm_channel.out_transfer(out_buffer.as_ref())?;
        }
        written
    };
    let csw = read_csw(comm_channel)?;
    if csw.tag != command.wrapper().tag {
        Err(ScsiError::from_cause(ErrorCause::ParseError))
    } else if csw.status != CommandStatusWrapper::COMMAND_PASSED {
        Err(ScsiError::from_cause(ErrorCause::FlagError {
            flags: csw.status as u32,
        }))
    } else {
        Ok((write, csw))
    }
}

fn transfer_in_command<Usb: CommunicationChannel, C: Command, InBuff: AsMut<[u8]>>(
    comm_channel: &mut Usb,
    command: &C,
    mut in_buffer: InBuff,
) -> Result<(usize, CommandStatusWrapper), ScsiError> {
    let _command_bytes = push_command(comm_channel, command)?;

    let transfer_length = command.wrapper().data_transfer_length;
    let read = if transfer_length == 0 {
        0
    } else if command.wrapper().direction == Direction::OUT {
        return Err(ScsiError::from_cause(ErrorCause::UnsupportedOperationError));
    } else {
        let mut read = comm_channel.in_transfer(in_buffer.as_mut())?;
        while read < transfer_length as usize {
            read += comm_channel.in_transfer(in_buffer.as_mut())?;
        }
        read
    };
    let csw = read_csw(comm_channel)?;
    if csw.tag != command.wrapper().tag {
        return Err(ScsiError::from_cause(ErrorCause::ParseError));
    } else if csw.status != CommandStatusWrapper::COMMAND_PASSED {
        return Err(ScsiError::from_cause(ErrorCause::FlagError {
            flags: csw.status as u32,
        }));
    }
    Ok((read, csw))
}
