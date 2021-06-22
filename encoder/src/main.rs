use std::fs;
use std::fs::File;
use std::path::Path;

use wav::header::WAV_FORMAT_PCM;
use wav::Header;

use hex2wav::create_audio_data;
use hex2wav::Options;

#[macro_use]
extern crate clap;
use clap::App;

fn main() {
    let options = get_options().unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(1);
    });

    let content = fs::read_to_string(&options.in_filename).expect("Could not load file");
    let out_path = Path::new(&options.out_filename);
    let mut out_file = File::create(&out_path).expect("could not open output file");

    let header = Header::new(WAV_FORMAT_PCM, 1, options.sample_rate, 8);
    let data = create_audio_data(content, &options);

    wav::write(header, &data.into(), &mut out_file).expect("could not write wav");

    println!("Wrote {}", out_path.display());
}

fn get_options() -> Result<Options, String> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let in_filename = matches.value_of("INPUT_FILE").unwrap().to_string();
    let out_filename = matches.value_of("OUTPUT_FILE").unwrap().to_string();

    let frame_size = matches
        .value_of("frame_size")
        .unwrap()
        .parse::<u16>()
        .map_err(|_| "Frame size should be a number")?;

    let cuttoff = matches
        .value_of("cuttoff")
        .unwrap()
        .parse::<u32>()
        .map_err(|_| "Cuttoff frequency should be a number")?;

    let sample_rate = matches
        .value_of("sample_rate")
        .unwrap()
        .parse::<u32>()
        .map_err(|_| "Sample rate should be a number")?;

    Ok(Options {
        in_filename,
        out_filename,
        frame_size,
        cuttoff,
        sample_rate,
    })
}
