#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(asm_experimental_arch)]
#![feature(asm_const)]

use arduino_hal::prelude::*;
use bootloader_rs::crc;

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    ufmt::uwriteln!(&mut serial, "Start\r").void_unwrap();

    let mut crc1: u16 = 0;
    let mut crc2: u16 = 0;

    for x in 0..1000000 {
        crc1 = crc_xmodem_update(crc1, x as u8);
        crc2 = crc_xmodem_update_asm(crc2, x as u8);

        if crc1 != crc2 {
            ufmt::uwriteln!(
                &mut serial,
                "Fail! at step {}, crc1: {}, crc2: {}",
                x,
                crc1,
                crc2
            )
            .void_unwrap();
        }
    }

    ufmt::uwriteln!(&mut serial, "PASS! crc1: {}, crc2: {}", crc1, crc2).void_unwrap();

    loop {}
}

#[inline(never)]
fn crc_xmodem_update(crc: u16, data: u8) -> u16 {
    crc::crc_xmodem_update(crc, data)
}

#[inline(never)]
fn crc_xmodem_update_asm(crc: u16, data: u8) -> u16 {
    crc::crc_xmodem_update_asm(crc, data)
}
