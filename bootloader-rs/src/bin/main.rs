#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(asm_experimental_arch)]
#![feature(asm_const)]

use bootloader_rs::frame::{FrameReceiver, FrameWriter};
use bootloader_rs::run_program;
use bootloader_rs::signal::SignalReceiver;
use bootloader_rs::spi::WriteOnlySpi;

use panic_halt as _;

/// # Safety
/// it's main, it can do what it wants
#[no_mangle]
unsafe extern "C" fn main() -> ! {
    let dp = atmega_hal::Peripherals::steal();
    let pins = atmega_hal::pins!(dp);
    let button = pins.pd0.into_pull_up_input();

    let signal_receiver = SignalReceiver::new(pins.pc3.into_pull_up_input(), dp.ADC, dp.AC, dp.TC0);
    let mut frame_receiver = FrameReceiver::new(signal_receiver);
    let writer = FrameWriter::new();

    let mut spi = WriteOnlySpi::new(
        dp.SPI,
        pins.pb5.into_output(),
        pins.pb3.into_output(),
        pins.pb2.into_output(),
    );
    
    spi.transmit(&[0xff, 0xff, 0xff, 0xff]);
    if button.is_high() {
        spi.transmit(&[0x00, 0x00, 0x00, 0x00]);
        run_program();
    }

    let mut pattern: u8 = 0b11;
    while let Some(frame) = frame_receiver.receive_frame() {
        if frame.is_run() {
            spi.transmit(&[0x00, 0x00, 0x00, 0x00]);
            run_program();
        }

        writer.write(frame);

        spi.transmit(&[pattern]);
        pattern <<= 1;
        if pattern == 0 {
            pattern = 0b11;
        }
    }

    spi.transmit(&[0x00, 0x00, 0x0f, 0xff]);

    loop {
        avr_device::asm::sleep();
    }
}