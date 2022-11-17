mod frame;
mod manchester_encoder;

pub use frame::Frame;
pub use manchester_encoder::ManchesterEncoder;

use ihex::{Reader, Record};
use std::iter;

pub fn find_page_size(mcu_name: &str) -> Option<usize> {
    let mcu = avr_mcu::microcontroller(mcu_name);

    mcu.device
        .address_spaces
        .iter()
        .find(|space| space.name == "prog")
        .and_then(|space| space.segments.first())
        .and_then(|seg| seg.page_size)
        .map(|size| size as usize)
}

pub fn create_audio_data(content: String, page_size: usize) -> Vec<u8> {
    let firmware_bytes = get_firmware_bytes(content);
    let frames =
        Frame::bytes_to_frames(&firmware_bytes, page_size).chain(iter::once(Frame::run(page_size)));

    let encoder = ManchesterEncoder::new();
    let audio_bytes = frames
        .scan(encoder, |encoder, frame| Some(encoder.encode_frame(frame)))
        .flatten();
    audio_bytes.collect()
}

fn get_firmware_bytes(content: String) -> Vec<u8> {
    let hex_reader = Reader::new(&content);

    hex_reader
        .into_iter()
        .flat_map(|r| r.ok())
        .fold(Vec::new(), |mut v: Vec<u8>, r| {
            if let Record::Data { offset, mut value } = r {
                while (v.len() as u16) < offset {
                    v.push(0xff);
                }
                v.append(&mut value);
            }

            v
        })
}
