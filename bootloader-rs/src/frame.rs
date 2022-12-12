use crate::crc;
use avr_boot::DataPage;

use core::mem::MaybeUninit;

const RUNCOMMAND: u8 = 3;

#[repr(C)]
pub struct Frame {
    pub command: u8,
    pub page_address: u16,
    page: DataPage,
    pub checksum: u16,
}

impl Frame {
    pub fn is_run(&self) -> bool {
        self.command == RUNCOMMAND
    }
}
pub trait ByteSource {
    fn sync(&mut self);
    fn get(&self) -> u8;
}

pub struct FrameReceiver<B: ByteSource> {
    signal_receiver: B,
    buffer: MaybeUninit<Frame>,
}

impl<B: ByteSource> FrameReceiver<B> {
    pub fn new(receiver: B) -> FrameReceiver<B> {
        FrameReceiver {
            signal_receiver: receiver,
            buffer: MaybeUninit::uninit(),
        }
    }

    pub fn receive_frame(&mut self) -> Option<&Frame> {
        let mut crc = 0u16;
        let mut i: u8 = core::mem::size_of::<Frame>() as u8 - 2;

        self.signal_receiver.sync();

        for uninit_byte in self.buffer.as_bytes_mut() {
            let init_byte = uninit_byte.write(self.signal_receiver.get());

            if i != 0 {
                crc = crc::crc_xmodem_update_asm(crc, *init_byte);
                i -= 1;
            }
        }
        // Safety: we have just written to every byte in the struct
        let frame: &Frame = unsafe { self.buffer.assume_init_ref() };

        if frame.checksum == crc {
            Some(frame)
        } else {
            None
        }
    }
}

pub struct FrameWriter {}

impl FrameWriter {
    pub fn new() -> FrameWriter {
        FrameWriter {}
    }

    pub fn write(&self, frame: &Frame) {

        let buff = avr_boot::PageBuffer::new(frame.page_address);
        buff.store_from(&frame.page);
    }
}

impl Default for FrameWriter {
    fn default() -> Self {
        Self::new()
    }
}