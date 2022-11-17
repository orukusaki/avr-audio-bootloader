use crate::crc;
use crate::spm;

use const_env__value::value_from_env;
use core::mem::MaybeUninit;

const SPM_PAGESIZE: usize = value_from_env!("SPM_PAGESIZE": usize);
const RUNCOMMAND: u8 = 3;

#[repr(C)]
pub struct Frame {
    pub command: u8,
    pub page_address: u16,
    page: [u16; SPM_PAGESIZE / 2],
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
        self.signal_receiver.sync();

        let mut crc = 0u16;
        let mut crc1 = 0u16;
        let mut crc2 = 0u16;

        for uninit_byte in self.buffer.as_bytes_mut() {
            let init_byte = uninit_byte.write(self.signal_receiver.get());

            // yes I know this looks silly, it's smaller than enumerating and an if-statement and I
            // need every byte
            crc2 = crc1;
            crc1 = crc;
            crc = crc::crc_xmodem_update_asm(crc, *init_byte);
        }
        // Safety: we have just written to every byte in the structure
        let frame: &Frame = unsafe { self.buffer.assume_init_ref() };

        if frame.checksum == crc2 {
            Some(frame)
        } else {
            None
        }
    }
}

pub struct FrameWriter {
    pgm: spm::Writer,
}

impl FrameWriter {
    pub fn new(pgm: spm::Writer) -> FrameWriter {
        FrameWriter {pgm}
    }

    pub fn write(&self, frame: &Frame) {
        self.pgm.page_erase(frame.page_address);

        let mut write_address = frame.page_address;
        for w in frame.page.iter() {
            self.pgm.page_fill(write_address, *w);
            write_address += 2;
        }

        self.pgm.page_write(frame.page_address);
        self.pgm.rww_enable();
    }
}
