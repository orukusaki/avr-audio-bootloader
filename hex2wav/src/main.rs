use std::fs;
use std::fs::File;
use std::path::Path;

use wav::header::WAV_FORMAT_PCM;
use wav::Header;

use clap::Parser;

use hex2wav::{create_audio_data, find_page_size};

fn main() {
    let options = Options::parse();

    let content = fs::read_to_string(&options.input_file).expect("Could not load file");
    let out_path = Path::new(&options.output_file);
    let mut out_file = File::create(&out_path).expect("could not open output file");
    let page_size =
        find_page_size(&options.mcu_name).expect("MCU not found");
    println!("Found page size for {}: {}", &options.mcu_name, page_size);

    let header = Header::new(WAV_FORMAT_PCM, 1, options.sample_rate, 8);
    let data = create_audio_data(content, page_size);
    wav::write(header, &data.into(), &mut out_file).expect("could not write wav");

    println!("Wrote {}", out_path.display());
}

/// Converts an intel .hex firmware file into a .wav using differential manchester encoding
#[derive(Parser, Debug)]
#[command(name = "hex2wav")]
#[command(author = "Peter Smith <peter@orukusaki.co.uk>")]
#[command(version = "0.1.0")]
#[command(about = "Converts an intel .hex firmware file into a .wav using differential manchester encoding", long_about = None)]
pub struct Options {
    /// Input file, should be in intel .hex format
    #[arg()]
    pub input_file: String,

    /// Output file
    #[arg(default_value = "firmware.wav")]
    pub output_file: String,

    // #[arg(short, long, default_value_t = 128)]
    // pub page_size: usize,
    /// MCU name, used to set the correct pgm page size
    #[arg(short, long, default_value = "atmega328p")]
    pub mcu_name: String,

    #[arg(short, long, default_value_t = 44000)]
    pub sample_rate: u32,
}
