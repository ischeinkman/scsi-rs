use scsi::commands::{InquiryCommand, InquiryResponse};
use scsi::commands::Read10Command;
use scsi::commands::{ReadCapacityCommand, ReadCapacityResponse};
use scsi::commands::TestUnitReady;
use scsi::commands::Write10Command;
use scsi::commands::{Command, CommandStatusWrapper, Direction};
use traits::{Buffer, BufferPullable, CommunicationChannel};
use error::{ScsiError, ErrorCause, UsbTransferDirection};

/// A struct that provides a simple, block-device-like interface around an SCSI device.
/// This allows for reading and writing to the device at static offests, allowing for
/// easy interaction with any file system crate.
pub struct ScsiBlockDevice<
    CommType: CommunicationChannel,
    BuffTypeA: Buffer,
    BuffTypeB: Buffer,
    BuffTypeC: Buffer,
> {
    comm_channel: CommType,
    in_buffer: BuffTypeA,
    out_buffer: BuffTypeB,
    scratch_buffer: BuffTypeC,
    block_size: u32,
}

impl<CommType: CommunicationChannel, BuffTypeA: Buffer, BuffTypeB: Buffer, BuffTypeC: Buffer>
    ScsiBlockDevice<CommType, BuffTypeA, BuffTypeB, BuffTypeC>
{
    /// Constructs a new `ScsiBlockDevice`.
    /// # Parameters
    /// *  `comm_channel` is the communication channel to be used to send out commands and read the responses.  
    /// *  `in_buffer` is a buffer that will be used to read in a response from `comm_channel`.
    /// *  `out_buffer` is a buffer that will be used to buffer SCSI commands before they are sent over `comm_channel`.
    /// *  `scratch_buffer` is a buffer that will be used for other, misc allocations, such as `CommandStatusWrapper` reading or setup commands.
    pub fn new(
        mut comm_channel: CommType,
        mut in_buffer: BuffTypeA,
        mut out_buffer: BuffTypeB,
        mut scratch_buffer: BuffTypeC,
    ) -> Result<Self, ScsiError> {
        let remaining = in_buffer.capacity() - in_buffer.size();

        let inquiry = InquiryCommand::new(remaining.min(36) as u8);
        let (_ir, _iw) = transfer_command_raw(
            &mut comm_channel,
            &inquiry,
            &mut in_buffer,
            &mut out_buffer,
            &mut scratch_buffer,
        )?;
        let inquiry_resp = InquiryResponse::pull_from_buffer(&mut in_buffer)?;
        if inquiry_resp.device_qualifier != 0 || inquiry_resp.device_type != 0 {
            return Err(ScsiError::from_cause(ErrorCause::InvalidDeviceError));
        }

        let test_unit = TestUnitReady::new();
        let (_tr, _tw) = transfer_command_raw(
            &mut comm_channel,
            &test_unit,
            &mut in_buffer,
            &mut out_buffer,
            &mut scratch_buffer,
        )?;

        let read_capacity = ReadCapacityCommand::new();
        let (_rr, _rw) = transfer_command_raw(
            &mut comm_channel,
            &read_capacity,
            &mut in_buffer,
            &mut out_buffer,
            &mut scratch_buffer,
        )?;
        let capacity_resp = ReadCapacityResponse::pull_from_buffer(&mut in_buffer)?;
        let block_size = capacity_resp.block_length;
        let rval = ScsiBlockDevice {
            comm_channel,
            in_buffer,
            out_buffer,
            scratch_buffer,
            block_size,
        };
        Ok(rval)
    }

    /// Reads bytes starting at `offset` into the provided `dest` buffer, returning
    /// the number of bytes read on success.
    pub fn read<B: Buffer>(&mut self, offset: u32, dest: &mut B) -> Result<usize, ScsiError> {
        let remaining = dest.capacity() - dest.size();
        if remaining % self.block_size as usize != 0 {
            return Err(ScsiError::from_cause(
                ErrorCause::NonBlocksizeMultipleLengthError{actual : remaining, block_size : self.block_size as usize},
            ));
        }
        let read_command = Read10Command::new(offset, remaining as u32, self.block_size)?;
        transfer_command_raw(
            &mut self.comm_channel,
            &read_command,
            dest,
            &mut self.out_buffer,
            &mut self.scratch_buffer,
        ).map(|(r, _)| r)
    }

    /// Writes bytes starting at `offset` from the provided buffer `src`, returning the
    /// number of bytes written on success.
    pub fn write<B: Buffer>(&mut self, offset: u32, src: &mut B) -> Result<usize, ScsiError> {
        let remaining = src.capacity() - src.size();
        if remaining % self.block_size as usize != 0 {
            return Err(ScsiError::from_cause(
                ErrorCause::NonBlocksizeMultipleLengthError{actual : remaining, block_size : self.block_size as usize},
            ));
        }
        let write_command = Write10Command::new(offset, remaining as u32, self.block_size)?;
        transfer_command_raw(
            &mut self.comm_channel,
            &write_command,
            &mut self.in_buffer,
            src,
            &mut self.scratch_buffer,
        ).map(|(r, _)| r)
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }
}

fn read_csw<C: CommunicationChannel, B: Buffer>(
    comm_channel: &mut C,
    scratch_buffer: &mut B,
) -> Result<CommandStatusWrapper, ScsiError> {
    if scratch_buffer.capacity() - scratch_buffer.size() < CommandStatusWrapper::SIZE as usize {
        return Err(ScsiError::from_cause(ErrorCause::BufferTooSmallError{expected : CommandStatusWrapper::SIZE as usize, actual : scratch_buffer.capacity() - scratch_buffer.size()}));
    }

    let read_count = comm_channel.in_transfer(scratch_buffer)?;
    if read_count != CommandStatusWrapper::SIZE as usize {
        return Err(ScsiError::from_cause(ErrorCause::UsbTransferError{direction : UsbTransferDirection::In}));
    }
    let retval = CommandStatusWrapper::pull_from_buffer(scratch_buffer)?;
    if retval.status != CommandStatusWrapper::COMMAND_PASSED as u8 {
        Err(ScsiError::from_cause(ErrorCause::FlagError))
    } else {
        Ok(retval)
    }
}

fn push_command<C: CommunicationChannel, Cmd: Command, B: Buffer>(
    comm_channel: &mut C,
    command: &Cmd,
    scratch_buffer: &mut B,
) -> Result<usize, ScsiError> {
    let serial_bytes = command.push_to_buffer(scratch_buffer)?;
    let pushed_bytes = comm_channel.out_transfer(scratch_buffer)?;
    if pushed_bytes != serial_bytes {
        Err(ScsiError::from_cause(ErrorCause::UsbTransferError{direction : UsbTransferDirection::Out}))
    } else {
        Ok(pushed_bytes)
    }
}

fn transfer_command_raw<
    Usb: CommunicationChannel,
    C: Command,
    InBuff: Buffer,
    OutBuff: Buffer,
    Scratch: Buffer,
>(
    comm_channel: &mut Usb,
    command: &C,
    in_buffer: &mut InBuff,
    out_buffer: &mut OutBuff,
    scratch_buffer: &mut Scratch,
) -> Result<(usize, usize), ScsiError> {
    let command_bytes = push_command(comm_channel, command, scratch_buffer)?;

    let transfer_length = command.wrapper().data_transfer_length;
    let (read, write) = if transfer_length > 0 {
        match command.wrapper().direction {
            Direction::IN => {
                let mut read = comm_channel.in_transfer(in_buffer)?;
                while read < transfer_length as usize {
                    read += comm_channel.in_transfer(in_buffer)?;
                }
                (read, 0)
            }
            _ => {
                let mut written = comm_channel.out_transfer(out_buffer)?;
                while written < transfer_length as usize {
                    written += comm_channel.out_transfer(out_buffer)?;
                }
                (0, written)
            }
        }
    } else {
        (0, 0)
    };
    let csw = read_csw(comm_channel, scratch_buffer)?;
    if csw.tag != command.wrapper().tag {
        return Err(ScsiError::from_cause(ErrorCause::FlagError));
    }
    Ok((
        read + CommandStatusWrapper::SIZE as usize,
        write + command_bytes,
    ))
}
