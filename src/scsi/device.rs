use traits::{CommunicationChannel, Buffer, BufferPullable};
use scsi::commands::{Command, CommandStatusWrapper, Direction};
use scsi::commands::read10::{Read10Command};
use scsi::commands::write10::{Write10Command};
use scsi::commands::inquiry::{InquiryCommand, InquiryResponse};
use scsi::commands::testunit::{TestUnitReady};
use scsi::commands::readcapacity::{ReadCapacityCommand, ReadCapacityResponse};
use {AumsError, ErrorCause};

pub struct ScsiBlockDevice<CommType : CommunicationChannel, BuffTypeA : Buffer, BuffTypeB : Buffer, BuffTypeC : Buffer> {
    comm_channel : CommType, 
    in_buffer : BuffTypeA, 
    out_buffer : BuffTypeB, 
    scratch_buffer : BuffTypeC, 
    block_size : u32, 
}

impl <CommType : CommunicationChannel, BuffTypeA : Buffer, BuffTypeB : Buffer, BuffTypeC : Buffer> ScsiBlockDevice<CommType, BuffTypeA, BuffTypeB, BuffTypeC> {
    pub fn new(mut comm_channel : CommType, mut in_buffer : BuffTypeA, mut out_buffer : BuffTypeB, mut scratch_buffer : BuffTypeC) -> Result<Self, AumsError> {
        let remaining = in_buffer.capacity() - in_buffer.size();
        
        let inquiry = InquiryCommand::new(remaining.min(36) as u8);
        let (_ir, _iw) = transfer_command_raw(&mut comm_channel, &inquiry, &mut in_buffer, &mut out_buffer, &mut scratch_buffer)?;
        let inquiry_resp = InquiryResponse::pull_from_buffer(&mut in_buffer)?;
        if inquiry_resp.device_qualifier != 0 || inquiry_resp.device_type != 0 {
            return Err(AumsError::from_cause(ErrorCause::InvalidInputError));
        }

        let test_unit = TestUnitReady::new();
        let (_tr, _tw) = transfer_command_raw(&mut comm_channel, &test_unit, &mut in_buffer, &mut out_buffer, &mut scratch_buffer)?;

        let read_capacity = ReadCapacityCommand::new();
        let (_rr, _rw) = transfer_command_raw(&mut comm_channel, &read_capacity, &mut in_buffer, &mut out_buffer, &mut scratch_buffer)?;
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

    pub fn read<B : Buffer>(&mut self, offset : u32, dest : &mut B) -> Result<usize, AumsError> {
        let remaining = dest.capacity() - dest.size(); 
        if remaining % self.block_size as usize != 0 {
            return Err(AumsError::from_cause(ErrorCause::InvalidInputError));
        }
        let read_command = Read10Command::new(offset , remaining as u32, self.block_size)?;
        transfer_command_raw(&mut self.comm_channel, &read_command, dest, &mut self.out_buffer, &mut self.scratch_buffer).map(|(r, _)| r)
    }

    pub fn write<B : Buffer>(&mut self, offset : u32, src : &mut B) -> Result<usize, AumsError> {
        let remaining = src.capacity() - src.size();
        if remaining % self.block_size as usize != 0 {
            return Err(AumsError::from_cause(ErrorCause::InvalidInputError));
        }
        let write_command = Write10Command::new(offset , remaining as u32, self.block_size)?;
        transfer_command_raw(&mut self.comm_channel, &write_command, &mut self.in_buffer, src, &mut self.scratch_buffer).map(|(r, _)| r)
    }

}

fn read_csw<C : CommunicationChannel, B : Buffer>(comm_channel : &mut C, scratch_buffer : &mut B) -> Result<CommandStatusWrapper, AumsError> {
    if scratch_buffer.capacity() - scratch_buffer.size() < CommandStatusWrapper::SIZE as usize {
        return Err(AumsError::from_cause(ErrorCause::InvalidInputError));
    }

    let read_count = comm_channel.in_transfer(scratch_buffer)?;
    if read_count != CommandStatusWrapper::SIZE as usize {
        return Err(AumsError::from_cause(ErrorCause::UsbTransferError));
    }
    let retval = CommandStatusWrapper::pull_from_buffer(scratch_buffer)?;
    if retval.status != CommandStatusWrapper::COMMAND_PASSED as u8 {
        Err(AumsError::from_cause(ErrorCause::FlagError))
    }
    else {
        Ok(retval)
    }
}

fn push_command<C : CommunicationChannel, Cmd : Command, B : Buffer> (comm_channel : &mut C, command : &Cmd, scratch_buffer : &mut B) -> Result<usize, AumsError> {
    let serial_bytes = command.push_to_buffer(scratch_buffer)?;
    let pushed_bytes = comm_channel.out_transfer(scratch_buffer)?;
    if pushed_bytes != serial_bytes {
        Err(AumsError::from_cause(ErrorCause::UsbTransferError))
    }
    else {
        Ok(pushed_bytes)
    }
    
}

fn transfer_command_raw<Usb : CommunicationChannel, C : Command, InBuff : Buffer, OutBuff : Buffer, Scratch : Buffer>(comm_channel : &mut Usb, command : &C, in_buffer : &mut InBuff, out_buffer : &mut OutBuff, scratch_buffer : &mut Scratch) -> Result<(usize, usize), AumsError> {
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
            },
            _ => {
                let mut written = comm_channel.out_transfer(out_buffer)?;
                while written < transfer_length as usize {
                    written += comm_channel.out_transfer(out_buffer)?; 
                }
                (0, written)
            }
        }
    } else { (0, 0) };
    let csw = read_csw(comm_channel, scratch_buffer)?;
    if csw.tag != command.wrapper().tag {
        return Err(AumsError::from_cause(ErrorCause::ParseError));
    }
    Ok((read + CommandStatusWrapper::SIZE as usize, write + command_bytes))
}