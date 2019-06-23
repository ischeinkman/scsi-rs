use crate::scsi::commands::{
    Command, CommandBlockWrapper, CommandStatusWrapper, InquiryCommand, InquiryResponse,
    Read10Command, ReadCapacityCommand, ReadCapacityResponse, RequestSenseCommand, TestUnitReady,
    Write10Command,
};
use crate::{
    BufferPullable, BufferPushable, CommunicationChannel, ErrorCause, ScsiError,
    UsbTransferDirection,
};

/// A trait to describe a device to respond to SCSI command, such as a flash drive.
///
/// This trait allows for more easily creating new SCSI-speaking devices by mapping
/// each command into 1 or 2 type-checking functions and then handling the command-independent
/// boiler plate in the included `process_command` function. Each function is passed the command
/// that triggered it, even if that command contains no information (such as TestUnitReady or RequestSense).
/// All SCSI sessions end with the device sending a `CommandStatusWrapper` back to the host; therefore,
/// nearly all functions here expect the device to construct one to be returned. Usually this will be done
/// via `Ok(CommandStatsWrapper::default())`, but error situations can be set as necessary as well.
pub trait ScsiResponder {
    /// The type to use as a memory buffer for per-block operations, mainly read
    /// or write transfers. Usually this would be of the form `[u8 ; N]`, where
    /// `N` is a multiple of 256.
    type BlockType: AsRef<[u8]> + AsMut<[u8]>;

    /// Called in response to a `ReadCapacityCommand` from the host.
    ///
    /// Since the command itself doesn't actually carry information, it should
    /// generally be ignored. However, it is still passed in just in case.
    fn read_capacity(
        &mut self,
        command: ReadCapacityCommand,
    ) -> Result<(ReadCapacityResponse, CommandStatusWrapper), ScsiError>;

    /// Called in response to a `InquiryCommand` from the host.
    ///
    /// Currently, the library does not yet include support for `allocation_length`s
    /// not equal to 36; in the future more research will be done into which fields
    /// are added and removed at different values.
    fn inquiry(
        &mut self,
        command: InquiryCommand,
    ) -> Result<(InquiryResponse, CommandStatusWrapper), ScsiError>;

    /// Called in response to a `InquiryCommand` from the host.
    ///
    /// Currently it is not known what the command's `allocation_length` field does,
    /// but it is still passed in to the command regardless.
    fn request_sense(
        &mut self,
        command: RequestSenseCommand,
    ) -> Result<CommandStatusWrapper, ScsiError>;

    /// Called in response to a `TestUnitReady` from the host.
    ///
    /// All of the response information is encoded in the `CommandStatusWrapper`;
    /// a responder that never fails should use `Ok(CommandStatusWrapper::default())`.
    fn test_unit_ready(
        &mut self,
        command: TestUnitReady,
    ) -> Result<CommandStatusWrapper, ScsiError>;

    /// Called when the host sends the `Read10` command itself over the
    /// wire.
    ///
    /// The responder should prepare for the upcoming `read_block` calls; this
    /// could involve things like pre-loading the relevant sections into RAM, setting
    /// up indices, etc.
    fn read10_start(&mut self, command: Read10Command) -> Result<(), ScsiError>;

    /// Called multiple times after a `read10_start` command to pull the relevant data out of the responder.
    ///
    /// `buffer` will be guranteed to be equal to the block length of the device as specified by the responder's
    /// `memory_buffer` method; in nearly all cases, it will be the same buffer. The method will keep being called until
    /// it returns `Some(_)`, even if this leads to a different number of blocks being read than expected by the original
    /// `Read10Command`; it is up to the responder to gurantee that the number of blocks read is correct.
    fn read_block(&mut self, buffer: &mut [u8]) -> Result<Option<CommandStatusWrapper>, ScsiError>;

    /// Called when the host sends the `Write10` command itself over the
    /// wire.
    ///
    /// The responder should prepare for the upcoming `write_block` calls; this
    /// could involve things like pre-loading the relevant sections into RAM, setting
    /// up indices, etc.
    fn write10_start(&mut self, command: Write10Command) -> Result<(), ScsiError>;

    /// Called multiple times after a `write10_start` command to pull the relevant data out of the responder.
    ///
    /// `buffer` will be guranteed to be equal to the block length of the device as specified by the responder's
    /// `memory_buffer` method; in nearly all cases, it will be the same buffer. The method will keep being called until
    /// it returns `Some(_)`, even if this leads to a different number of blocks being written to than expected by the original
    /// `Write10Command`; it is up to the responder to gurantee that the number of blocks read is correct.
    fn write_block(&mut self, buffer: &[u8]) -> Result<Option<CommandStatusWrapper>, ScsiError>;

    /// Generates a new, owned instance of the responder's block buffer.
    ///
    /// Usually, this can be implemented as just `[0 ; N]`, where `N` is the same
    /// as the one picked for `Self::BlockType`.
    fn memory_buffer(&mut self) -> Self::BlockType;

    /// Processes a single command from a host, from reading the CBW to outputting
    /// the CSW.
    ///
    /// First, the CBW and command header is read from `channel` via `in_transfer`;
    /// the correct method is then called on `self` based on which opcode was read.
    /// Next, if necessary for that particular command (currently, only `Write10`),
    /// a new block buffer will be allocated via `self.memory_buffer()` and any needed input blocks
    /// will be pulled from `channel` and routed to the relevant method on `self`.
    /// Next, if necessary for that particular command (currently, only `Read10`),
    /// a new block buffer will be allocated via `self.memory_buffer()` and any needed ouput blocks
    /// will be pulled from the relevant method on `self` and pushed to `channel`.
    /// Next, if the command has an extra specialized response struct, it will be sent via `channel.out_transfer`
    /// using a 31-length buffer.
    /// Finally, the CSW struct's tag is set to match the input CBW's and it is sent across the channel using a 31-length buffer.
    fn process_command<C: CommunicationChannel>(
        &mut self,
        channel: &mut C,
    ) -> Result<(), ScsiError> {
        let mut command_buffer = [0; 31];
        let read = channel.in_transfer(&mut command_buffer)?;
        if read != 31 {
            return Err(ScsiError::from_cause(ErrorCause::UsbTransferError {
                direction: UsbTransferDirection::In,
            }));
        }
        let cbw = CommandBlockWrapper::pull_from_buffer(&command_buffer)?;
        let command = ScsiCommand::pull_from_buffer(&command_buffer)?;
        let mut csw: CommandStatusWrapper = match command {
            ScsiCommand::ReadCapacity(rcc) => {
                let (response, csw) = self.read_capacity(rcc)?;
                let _response_pushed = response.push_to_buffer(&mut command_buffer)?;
                let _response_sent = channel.out_transfer(&command_buffer)?;
                csw
            }
            ScsiCommand::Inquiry(ic) => {
                let (response, csw) = self.inquiry(ic)?;
                let _response_pushed = response.push_to_buffer(&mut command_buffer)?;
                let _response_sent = channel.out_transfer(&command_buffer)?;
                csw
            }
            ScsiCommand::RequestSense(rc) => self.request_sense(rc)?,
            ScsiCommand::TestUnitReady(tc) => self.test_unit_ready(tc)?,
            ScsiCommand::Read10(rten) => {
                self.read10_start(rten)?;
                let mut block = self.memory_buffer();
                let mut block_ref = block.as_mut();
                loop {
                    let csw_opt = self.read_block(&mut block_ref)?;
                    let _out_len = channel.out_transfer(&block_ref)?;
                    if let Some(csw) = csw_opt {
                        break csw;
                    }
                }
            }
            ScsiCommand::Write10(wten) => {
                self.write10_start(wten)?;
                let mut block = self.memory_buffer();
                let mut block_ref = block.as_mut();
                loop {
                    let _out_len = channel.in_transfer(&mut block_ref)?;
                    let csw_opt = self.write_block(&block_ref)?;
                    if let Some(csw) = csw_opt {
                        break csw;
                    }
                }
            } //_ => unimplemented!()
        };
        csw.tag = cbw.tag;
        let csw_pushed = csw.push_to_buffer(&mut command_buffer)?;
        if csw_pushed != 13 {
            return Err(ScsiError::from_cause(ErrorCause::BufferTooSmallError {
                expected: 13,
                actual: csw_pushed,
            }));
        }
        let csw_sent = channel.out_transfer(&command_buffer)?;
        if csw_sent != 31 {
            return Err(ScsiError::from_cause(ErrorCause::BufferTooSmallError {
                expected: 31,
                actual: csw_sent,
            }));
        }
        Ok(())
    }
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ScsiCommand {
    Inquiry(InquiryCommand),
    Read10(Read10Command),
    ReadCapacity(ReadCapacityCommand),
    RequestSense(RequestSenseCommand),
    TestUnitReady(TestUnitReady),
    Write10(Write10Command),
}

impl BufferPullable for ScsiCommand {
    fn pull_from_buffer<T: AsRef<[u8]>>(buffer: T) -> Result<Self, ScsiError> {
        let buffer = buffer.as_ref();
        let opcode = buffer[15];
        if opcode == InquiryCommand::opcode() {
            Ok(ScsiCommand::Inquiry(InquiryCommand::pull_from_buffer(
                buffer,
            )?))
        } else if opcode == Read10Command::opcode() {
            Ok(ScsiCommand::Read10(Read10Command::pull_from_buffer(
                buffer,
            )?))
        } else if opcode == ReadCapacityCommand::opcode() {
            Ok(ScsiCommand::ReadCapacity(
                ReadCapacityCommand::pull_from_buffer(buffer)?,
            ))
        } else if opcode == RequestSenseCommand::opcode() {
            Ok(ScsiCommand::RequestSense(
                RequestSenseCommand::pull_from_buffer(buffer)?,
            ))
        } else if opcode == TestUnitReady::opcode() {
            Ok(ScsiCommand::TestUnitReady(TestUnitReady::pull_from_buffer(
                buffer,
            )?))
        } else if opcode == Write10Command::opcode() {
            Ok(ScsiCommand::Write10(Write10Command::pull_from_buffer(
                buffer,
            )?))
        } else {
            Err(ScsiError::from_cause(ErrorCause::UnsupportedOperationError))
        }
    }
}

impl BufferPushable for ScsiCommand {
    fn push_to_buffer<T: AsMut<[u8]>>(&self, buffer: T) -> Result<usize, ScsiError> {
        match self {
            ScsiCommand::Inquiry(c) => c.push_to_buffer(buffer),
            ScsiCommand::Read10(c) => c.push_to_buffer(buffer),
            ScsiCommand::ReadCapacity(c) => c.push_to_buffer(buffer),
            ScsiCommand::RequestSense(c) => c.push_to_buffer(buffer),
            ScsiCommand::TestUnitReady(c) => c.push_to_buffer(buffer),
            ScsiCommand::Write10(c) => c.push_to_buffer(buffer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CommandStatusWrapper, CommunicationChannel, ErrorCause, InquiryCommand, InquiryResponse,
        Read10Command, ReadCapacityCommand, ReadCapacityResponse, RequestSenseCommand, ScsiError,
        ScsiResponder, TestUnitReady, Write10Command,
    };
    use std::sync::{Arc, Mutex};
    use std::vec::Vec;
    use traits::{BufferPullable, BufferPushable};

    struct BlockType([u8; 256]);
    impl AsRef<[u8]> for BlockType {
        fn as_ref(&self) -> &[u8] {
            &self.0
        }
    }
    impl AsMut<[u8]> for BlockType {
        fn as_mut(&mut self) -> &mut [u8] {
            &mut self.0
        }
    }

    struct TestResponder {
        buffer: [u8; 256 * 1024],
        read_cursor: usize,
        read_size: u16,
        write_cursor: usize,
        write_size: u16,
    }
    impl Default for TestResponder {
        fn default() -> Self {
            TestResponder {
                buffer: [0; 256 * 1024],
                read_cursor: 0,
                read_size: 0,
                write_cursor: 0,
                write_size: 0,
            }
        }
    }

    impl ScsiResponder for TestResponder {
        type BlockType = BlockType;
        fn read_capacity(
            &mut self,
            _command: ReadCapacityCommand,
        ) -> Result<(ReadCapacityResponse, CommandStatusWrapper), ScsiError> {
            let resp = ReadCapacityResponse {
                logical_block_address: 0,
                block_length: 256,
            };
            let csw = CommandStatusWrapper::default();
            Ok((resp, csw))
        }

        fn inquiry(
            &mut self,
            _command: InquiryCommand,
        ) -> Result<(InquiryResponse, CommandStatusWrapper), ScsiError> {
            Ok((InquiryResponse::default(), CommandStatusWrapper::default()))
        }
        fn request_sense(
            &mut self,
            _command: RequestSenseCommand,
        ) -> Result<CommandStatusWrapper, ScsiError> {
            Ok(CommandStatusWrapper::default())
        }
        fn test_unit_ready(
            &mut self,
            _command: TestUnitReady,
        ) -> Result<CommandStatusWrapper, ScsiError> {
            Ok(CommandStatusWrapper::default())
        }

        fn read10_start(&mut self, command: Read10Command) -> Result<(), ScsiError> {
            self.read_cursor = command.block_address as usize;
            self.read_size = command.transfer_blocks;
            Ok(())
        }
        fn read_block(
            &mut self,
            buffer: &mut [u8],
        ) -> Result<Option<CommandStatusWrapper>, ScsiError> {
            if self.read_size == 0 {
                return Ok(Some(CommandStatusWrapper::default()));
            }
            if buffer.len() != 256 {
                return Err(ScsiError::from_cause(ErrorCause::BufferTooSmallError {
                    expected: 256,
                    actual: buffer.len(),
                }));
            }
            let read_slice = &self.buffer[256 * self.read_cursor..256 * (self.read_cursor + 1)];
            (buffer).copy_from_slice(&read_slice);
            self.read_cursor += 1;
            self.read_size -= 1;
            Ok(None)
        }

        fn write10_start(&mut self, command: Write10Command) -> Result<(), ScsiError> {
            self.write_cursor = command.block_address as usize;
            self.write_size = command.transfer_blocks;
            Ok(())
        }
        fn write_block(
            &mut self,
            buffer: &[u8],
        ) -> Result<Option<CommandStatusWrapper>, ScsiError> {
            if self.write_size == 0 {
                return Ok(Some(CommandStatusWrapper::default()));
            }
            if buffer.len() != 256 {
                return Err(ScsiError::from_cause(ErrorCause::BufferTooSmallError {
                    expected: 256,
                    actual: buffer.len(),
                }));
            }
            let write_slice =
                &mut self.buffer[256 * self.write_cursor..256 * (self.write_cursor + 1)];
            write_slice.copy_from_slice(&buffer);
            self.write_cursor += 1;
            self.write_size -= 1;
            Ok(None)
        }

        fn memory_buffer(&mut self) -> Self::BlockType {
            BlockType([0; 256])
        }
    }

    #[derive(Clone, Default)]
    struct TestDualChannel {
        pub send_buff: Arc<Mutex<Vec<u8>>>,
        pub recv_buff: Arc<Mutex<Vec<u8>>>,
    }

    impl TestDualChannel {
        pub fn reversed(&self) -> TestDualChannel {
            TestDualChannel {
                send_buff: Arc::clone(&self.recv_buff),
                recv_buff: Arc::clone(&self.send_buff),
            }
        }

        pub fn clear(&mut self) {
            self.send_buff.lock().map(|mut v| v.clear()).unwrap();
            self.recv_buff.lock().map(|mut v| v.clear()).unwrap();
        }
    }

    impl CommunicationChannel for TestDualChannel {
        fn out_transfer<B: AsRef<[u8]>>(&mut self, bytes: B) -> Result<usize, ScsiError> {
            let slice = bytes.as_ref();
            let mut snd_lock = self
                .send_buff
                .lock()
                .map_err(|_e| ScsiError::from_cause(ErrorCause::UnsupportedOperationError))?;
            for itm in slice {
                snd_lock.push(*itm);
            }
            Ok(slice.len())
        }

        fn in_transfer<B: AsMut<[u8]>>(&mut self, mut buffer: B) -> Result<usize, ScsiError> {
            let slice = buffer.as_mut();
            let mut rcv_lock = self
                .recv_buff
                .lock()
                .map_err(|_e| ScsiError::from_cause(ErrorCause::UnsupportedOperationError))?;
            let buflen = rcv_lock.len();
            let mut read = 0;
            {
                let iter = rcv_lock.drain(..slice.len().min(buflen));
                for itm in iter {
                    slice[read] = itm;
                    read += 1;
                }
            }
            Ok(read)
        }
    }

    #[test]
    fn test_exchange() {
        let mut forward = TestDualChannel::default();
        let mut responder_side = forward.reversed();

        let mut dev = TestResponder::default();

        let mut command_buff = [0; 31];
        let capacity_req = ReadCapacityCommand::new();
        assert_eq!(16, capacity_req.push_to_buffer(&mut command_buff).unwrap());
        assert_eq!(31, forward.out_transfer(&command_buff).unwrap());
        assert_eq!(31, forward.send_buff.lock().unwrap().len());
        assert_eq!(31, responder_side.recv_buff.lock().unwrap().len());

        dev.process_command(&mut responder_side).unwrap();

        let (resp, csw) = {
            let buff_raw = forward.recv_buff.lock().unwrap();
            let buff: &Vec<u8> = buff_raw.as_ref();
            let resp = ReadCapacityResponse::pull_from_buffer(&buff).unwrap();
            let csw = CommandStatusWrapper::pull_from_buffer(&buff[31..]).unwrap();
            (resp, csw)
        };
        assert_eq!(256, resp.block_length);
        assert_eq!(CommandStatusWrapper::COMMAND_PASSED, csw.status);

        let block_buff: &[u8] = &[0xFF; 256];
        let write_a = Write10Command::new(0, 256, resp.block_length).unwrap();
        forward.clear();
        responder_side.clear();
        write_a.push_to_buffer(&mut command_buff).unwrap();
        forward.out_transfer(&command_buff).unwrap();
        forward.out_transfer(&block_buff).unwrap();

        dev.process_command(&mut responder_side).unwrap();

        assert_eq!(&dev.buffer[0..256], block_buff);

        let mut bbuff_2: &mut [u8] = &mut [0; 256];
        let read_a = Read10Command::new(0, 256, resp.block_length).unwrap();
        forward.clear();
        responder_side.clear();
        read_a.push_to_buffer(&mut command_buff).unwrap();
        forward.out_transfer(&command_buff).unwrap();

        dev.process_command(&mut responder_side).unwrap();

        assert_eq!(256, forward.in_transfer(&mut bbuff_2).unwrap());
        assert_eq!(&bbuff_2, &block_buff);
    }

}
