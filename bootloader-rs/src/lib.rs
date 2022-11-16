#![no_std]
#![feature(asm_experimental_arch)]
#![feature(asm_const)]
#![feature(maybe_uninit_as_bytes)]
#![feature(core_intrinsics)]

pub mod crc;
pub mod frame;
pub mod signal;
pub mod spi;
pub mod spm;

use core::arch::asm;
use core::hint::unreachable_unchecked;

pub fn run_program() -> ! {
    // SAFETY, this function does not return, so stealing the peripherals is ok
    let dp = unsafe { atmega_hal::Peripherals::steal() };

    // dp.PORTB.ddrb.reset();
    dp.PORTC.ddrc.reset();
    dp.PORTD.ddrd.reset();
    dp.TC1.tccr1b.reset();

    // restore interrupts
    dp.CPU
        .mcucr
        .write(|w| w.ivce().set_bit().ivsel().clear_bit());

    //Jump to address 0
    unsafe { asm!("jmp 0x0000") }
    unsafe { unreachable_unchecked() };
}
