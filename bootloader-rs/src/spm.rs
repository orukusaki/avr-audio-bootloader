use core::arch::asm;

use atmega_hal::pac::{cpu::spmcsr::SPMCSR_SPEC, CPU};

pub struct Writer {
    cpu: CPU,
}

impl Writer {
    pub fn new(cpu: CPU) -> Self {
        Writer { cpu }
    }

    pub fn page_fill(&self, address: u16, data: u16) {
        self.spm_with_address_and_data(|w| w.spmen().set_bit(), address, data);
    }

    pub fn page_erase(&self, address: u16) {
        self.spm_with_address(|w| w.spmen().set_bit().pgers().set_bit(), address);
    }

    pub fn page_write(&self, address: u16) {
        self.spm_with_address(|w| w.spmen().set_bit().pgwrt().set_bit(), address);
    }

    pub fn rww_enable(&self) {
        self.spm(|w| w.spmen().set_bit().rwwsre().set_bit());
    }

    fn spm_busy_wait(&self) {
        loop {
            if !self.spm_busy() {
                break;
            }
        }
    }

    fn spm_busy(&self) -> bool {
        self.cpu.spmcsr.read().spmen().bit_is_set()
    }

    /**
     *   Due to an issue in LLVM, the r0 and r1 registers cannot be used as inputs or outputs. If modified, they must be restored to their original values before the end of the block.
     */
    #[inline(always)]
    fn spm_with_address_and_data<F>(&self, f: F, address: u16, data: u16)
    where
        F: FnOnce(&mut atmega_hal::pac::cpu::spmcsr::W) -> &mut avr_device::generic::W<SPMCSR_SPEC>,
    {
        unsafe {
            asm! (
                "push r0",
                "movw  r0, {data}",
                data = in(reg_pair) data,
            );
            self.spm_with_address(f, address);
            asm!("eor r1, r1", "pop r0",)
        }
    }

    #[inline(always)]
    fn spm_with_address<F>(&self, f: F, address: u16)
    where
        F: FnOnce(&mut atmega_hal::pac::cpu::spmcsr::W) -> &mut avr_device::generic::W<SPMCSR_SPEC>,
    {
        unsafe {
            asm! (
                "",
                in("Z") address
            );
            self.spm(f);
        }
    }

    #[inline(always)]
    fn spm<F>(&self, f: F)
    where
        F: FnOnce(&mut atmega_hal::pac::cpu::spmcsr::W) -> &mut avr_device::generic::W<SPMCSR_SPEC>,
    {
        self.spm_busy_wait();
        self.cpu.spmcsr.write(f);
        unsafe { asm!("spm") }
    }
}
