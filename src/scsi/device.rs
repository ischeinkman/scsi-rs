use scsi::commands::{InquiryCommand, InquiryResponse};
use scsi::commands::Read10Command;
use scsi::commands::{ReadCapacityCommand, ReadCapacityResponse};
use scsi::commands::TestUnitReady;
use scsi::commands::Write10Command;
use scsi::commands::{Command, CommandStatusWrapper, Direction};
use traits::{BufferPullable, CommunicationChannel};
use error::{ScsiError, ErrorCause, UsbTransferDirection};

/// A struct that provides a simple, block-device-like interface around an SCSI device.
/// This allows for reading and writing to the device at static offests, allowing for
/// easy interaction with any file system crate.
pub struct ScsiBlockDevice<
    CommType: CommunicationChannel,
    BuffTypeA: AsMut<[u8]> + AsRef<[u8]>,
    BuffTypeB: AsMut<[u8]> + AsRef<[u8]>,
> {
    pub comm_channel: CommType,
    in_buffer: BuffTypeA,
    out_buffer: BuffTypeB,
    block_size: u32,

    pub prev_csw : Option<CommandStatusWrapper>,
}

impl<CommType: CommunicationChannel, BuffTypeA: AsMut<[u8]> + AsRef<[u8]>, BuffTypeB: AsMut<[u8]> + AsRef<[u8]>>
    ScsiBlockDevice<CommType, BuffTypeA, BuffTypeB>
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
    ) -> Result<Self, ScsiError> {
        zero_buffer(in_buffer.as_mut());
        zero_buffer(out_buffer.as_mut());
        let inquiry = InquiryCommand::new(in_buffer.as_ref().len().min(36) as u8);
        let (_ir, _iw, _csw_ic) = transfer_command_raw(
            &mut comm_channel,
            &inquiry,
            &mut in_buffer,
            &mut out_buffer,
        )?;
        let inquiry_resp = InquiryResponse::pull_from_buffer(&mut in_buffer)?;
        if inquiry_resp.device_qualifier != 0 || inquiry_resp.device_type != 0 {
            return Err(ScsiError::from_cause(ErrorCause::InvalidDeviceError));
        }

        let test_unit = TestUnitReady::new();
        zero_buffer(in_buffer.as_mut());
        zero_buffer(out_buffer.as_mut());
        let (_tr, _tw, _csw_tur) = transfer_command_raw(
            &mut comm_channel,
            &test_unit,
            &mut in_buffer,
            &mut out_buffer,
        )?;

        let read_capacity = ReadCapacityCommand::new();
        zero_buffer(in_buffer.as_mut());
        zero_buffer(out_buffer.as_mut());
        let (_rr, _rw, mut csw_rcc) = transfer_command_raw(
            &mut comm_channel,
            &read_capacity,
            &mut in_buffer,
            &mut out_buffer,
        )?;
        csw_rcc.tag = 2;
        let capacity_resp = ReadCapacityResponse::pull_from_buffer(in_buffer.as_ref())?;
        let block_size = capacity_resp.block_length;
        let rval = ScsiBlockDevice {
            comm_channel,
            in_buffer,
            out_buffer,
            block_size,
            prev_csw : Some(csw_rcc),
        };
        Ok(rval)
    }

    /// Reads bytes starting at `offset` into the provided `dest` buffer, returning
    /// the number of bytes read on success.
    pub fn read<B: AsMut<[u8]>>(&mut self, offset: u32, mut dest: B) -> Result<usize, ScsiError> {
        let buffer = dest.as_mut();
        let prev_tag = match &self.prev_csw {
            Some(ref c) => c.tag,
            None => 0
        };
        self.prev_csw = None;
        zero_buffer(self.out_buffer.as_mut());
        if buffer.len() == 0 {
            return Ok(0);
        }
        if buffer.len() % self.block_size as usize != 0 {
            return Err(ScsiError::from_cause(
                ErrorCause::NonBlocksizeMultipleLengthError{actual : buffer.len(), block_size : self.block_size as usize},
            ));
        }
        let read_command = Read10Command::new(offset, buffer.len() as u32, self.block_size)?;
        let (r, _, mut csw) = transfer_command_raw(
            &mut self.comm_channel,
            &read_command,
            buffer,
            &mut self.out_buffer,
        )?;
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
            None => 0
        };
        self.prev_csw = None;
        zero_buffer(self.in_buffer.as_mut());
        let to_transfer = buffer.len();
        if to_transfer % self.block_size as usize != 0 {
            return Err(ScsiError::from_cause(
                ErrorCause::NonBlocksizeMultipleLengthError{actual : to_transfer, block_size : self.block_size as usize},
            ));
        }
        let write_command = Write10Command::new(offset, to_transfer as u32, self.block_size)?;
        let (_, w, mut csw) = transfer_command_raw(
            &mut self.comm_channel,
            &write_command,
            &mut self.in_buffer,
            buffer,
        )?;
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
        return Err(ScsiError::from_cause(ErrorCause::UsbTransferError{direction : UsbTransferDirection::In}));
    }
    let retval = CommandStatusWrapper::pull_from_buffer(scratch_buffer)?;
    Ok(retval)
}

fn push_command<C: CommunicationChannel, Cmd: Command>(
    comm_channel: &mut C,
    command: &Cmd,
) -> Result<usize, ScsiError> {
    let scratch_buffer  = [0 ; 31];
    // Push the command's bytes to the buffer
    let _serial_bytes = command.push_to_buffer(scratch_buffer)?;
    let pushed_bytes = comm_channel.out_transfer(scratch_buffer)?;
    if pushed_bytes != 31 {
        Err(ScsiError::from_cause(ErrorCause::UsbTransferError{direction : UsbTransferDirection::Out}))
    } else {
        Ok(pushed_bytes)
    }
}

fn transfer_command_raw<
    Usb: CommunicationChannel,
    C: Command,
    InBuff: AsMut<[u8]>,
    OutBuff: AsRef<[u8]>,
>(
    comm_channel: &mut Usb,
    command: &C,
    mut in_buffer: InBuff,
    out_buffer: OutBuff,
) -> Result<(usize, usize, CommandStatusWrapper), ScsiError> {
    let _command_bytes = push_command(comm_channel, command)?;

    let transfer_length = command.wrapper().data_transfer_length;
    let (read, write) = if transfer_length > 0 {
        match command.wrapper().direction {
            Direction::IN => {
                let mut read = comm_channel.in_transfer(in_buffer.as_mut())?;
                while read < transfer_length as usize {
                    read += comm_channel.in_transfer(in_buffer.as_mut())?;
                }
                (read, 0)
            }
            _ => {
                let mut written = comm_channel.out_transfer(out_buffer.as_ref())?;
                while written < transfer_length as usize {
                    written += comm_channel.out_transfer(out_buffer.as_ref())?;
                }
                (0, written)
            }
        }
    } else {
        (0, 0)
    };
    let csw = read_csw(comm_channel)?;
    if csw.tag != command.wrapper().tag {
        return Err(ScsiError::from_cause(ErrorCause::ParseError));
    }
    else if csw.status != CommandStatusWrapper::COMMAND_PASSED {
        return Err(ScsiError::from_cause(ErrorCause::FlagError { flags : csw.status as u32}))
    }
    Ok((
        read,
        write,
        csw
    ))
}

fn zero_buffer<B : AsMut<[u8]>>(mut buffer :B) {
    let buffer = buffer.as_mut();
    for elm in buffer.iter_mut() {
        *elm = 0;
    }
}