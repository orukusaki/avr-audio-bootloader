#!/bin/sh
# export AVR_CPU_FREQUENCY_HZ=16000000
cargo build -Z build-std=core -p  bootloader-rs --target=avr-unknown-gnu-atmega328 --release --verbose
