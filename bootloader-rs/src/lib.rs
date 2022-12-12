#![no_std]
#![feature(asm_experimental_arch)]
#![feature(asm_const)]
#![feature(asm_sym)]
#![feature(maybe_uninit_as_bytes)]
#![feature(core_intrinsics)]

pub mod crc;
pub mod frame;
pub mod signal;
pub mod spi;

use core::arch::asm;

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
    unsafe { 
        asm!(
        "ijmp", 
        in("Z") 0u16,
        options(noreturn, nomem, nostack)
        ) 
    }
}
