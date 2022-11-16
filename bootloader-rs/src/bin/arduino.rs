#![no_std]
#![no_main]
#![feature(core_intrinsics)]

use arduino_hal::prelude::*;
use bootloader_rs::frame::FrameReceiver;
use bootloader_rs::signal::SignalReceiver;

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut led = pins.d13.into_output();

    let signal_receiver = SignalReceiver::new(pins.a3.into_pull_up_input(), dp.ADC, dp.AC, dp.TC0);
    let mut frame_receiver = FrameReceiver::new(signal_receiver);

    led.set_high();
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    ufmt::uwriteln!(&mut serial, "Start\r").void_unwrap();

    loop {
        if let Some(frame) = frame_receiver.receive_frame() {
            led.toggle();
            ufmt::uwriteln!(
                &mut serial,
                "{} {:#06x}, {:#06x}",
                frame.command,
                frame.page_address,
                frame.checksum
            )
            .void_unwrap();
        } else {
            ufmt::uwriteln!(&mut serial, "Bad frame\r").void_unwrap();
        }
    }
}
