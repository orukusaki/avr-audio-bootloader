mod frame;
pub mod lpf;
mod manchester_encoder;

pub use frame::Frame;
pub use manchester_encoder::ManchesterEncoder;

use ihex::{Reader, Record};
use std::iter;

pub struct Options {
    pub in_filename: String,
    pub out_filename: String,
    pub frame_size: u16,
    pub cuttoff: u32,
    pub sample_rate: u32,
}

pub fn create_audio_data(content: String, options: &Options) -> Vec<u8> {
    let firmware_bytes = get_firmware_bytes(content);
    let frames = Frame::bytes_to_frames(&firmware_bytes, options.frame_size.into())
        .chain(iter::once(Frame::run(options.frame_size.into())));

    let encoder = ManchesterEncoder::new();
    let audio_bytes = frames
        .scan(encoder, |encoder, frame| Some(encoder.encode_frame(frame)))
        .flatten();

    let filtered_audio = lpf::filter(
        audio_bytes,
        options.cuttoff as f32,
        options.sample_rate as f32,
    );

    filtered_audio.map(|s| s as u8).collect()
}

fn get_firmware_bytes(content: String) -> Vec<u8> {
    let hex_reader = Reader::new(&content);

    hex_reader
        .into_iter()
        .flat_map(|r| r.ok())
        .fold(Vec::new(), |mut v: Vec<u8>, r| {
            if let Record::Data { offset, mut value } = r {
                while (v.len() as u16) < offset {
                    v.push(0u8);
                }
                v.append(&mut value);
            }

            v
        })
}
