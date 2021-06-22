
# AVR audio bootloader
[![Build Status](https://travis-ci.com/orukusaki/avr-audio-bootloader.svg?branch=main)](https://travis-ci.com/orukusaki/avr-audio-bootloader)

A bootloader for the ATmega328P, which enables firmware updates via an audio signal, based on the work of Christoph Haberer. It takes less than 1K of flash memory.

Encode your project firmware using the included Rust program.  The resulting .wav file can be played using the headphone port of any laptop/phone etc into your project to update the firmware.

# Hardware

The analog comparator is used for accurate zero crossing detection.
Connect the audio input to `PC3 (A3)` via a 20k resistor. Connect `PC3` to `AIN0 (D6)` via a 100k resistor, and add a 100n capacitor from `AIN0` to `GND`.
Connect a push-switch between `PD0 (D0)` and `GND`.  Hold down the button while powering on the device to enter the bootloader.

# Building the bootloader

This project is configured using [PlatformIO](https://platformio.org/). Please see PlatformIO documentation for installation instructions. To build this project use:

    pio run

Two images are built. The "uno" build is not actually a bootloader - it's a normal firmware image used for debugging the data transfer on an Arduino Uno.  This build will not actually write any data to the program memory, but it will give debugging info over the console monitor.

The "atmega328p" build is the real bootloader image, designed to run on a standalone chip. Feedback is provided by a string of 32 Leds connected to shift-registers attached to the SPI port.  If your project isn't set up the same way, you'll probably need to create your own custom build.
You will need to upload the bootloader using an In System Programmer.  Config is provided for using a "usbasp" device, which are available cheaply online.  The bootloader will not overwrite itself, so if you need to modify it, you will need to use the ISP.
You can also fine a pre-built bootloader image in the releases page on github.

# Encoding your project firmware

On the Releases page, you should a compiled binary for your operating system of choice, just download and unpack the appropriate archive.
Alternatively you can build from source, install Rust using [rustup](https://rustup.rs/), then

    cargo build --release

## Usage
    hex2wav [OPTIONS] <INPUT_FILE> [OUTPUT_FILE]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -c, --cuttoff <FREQ>              low pass filter cuttoff Hz [default: 10000]
        -f, --framesize <SIZE>            Frame size in bytes [default: 128]
        -s, --samplerate <SAMPLE_RATE>    Output file sample rate [default: 48000]

    ARGS:
        <INPUT_FILE> Input file, should be in intel .hex format
        <OUTPUT_FILE>  Output .wav file [default: firmware.wav]

Pass in the name of your built firmware image, e.g:

    hex2wav ~/Projects/myProject/.pio/uno/firmware.hex

You will now have a `firmware.wav` file which you can play using any media player.

# Customising

Your project probably has a different hardware setup to mine, so I've tried to make it easy to add your own customisations in.  Start by copying the `src/env/atmega328p/` folder, and the relevant section in `platformio.ini`.  Watch the final image size - if it gets above 1K, you'll need to change the start address and the fuses.  Both of these things are configured in the platformio.ini file. Check out the boot loader section of the Atmega328 datasheet for more details on how to set these correctly.  

# License
Please see attached LICENSE file

# Contributing
PRs are welcome on [github](https://github.com/orukusaki/avr-audio-bootloader).  Things I'd be particularly interested in:
* Configurations for other build systems (AVR studio, VSCode etc)
* Improvements to the extensibility of the core
* Adaptions for other MCUs
* Expanding the .hex parsing to handle more record types
* Support for encoding other file types, such as .elf and .bin
