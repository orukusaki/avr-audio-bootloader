#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(maybe_uninit_as_bytes)]
#![feature(asm_experimental_arch)]
#![feature(asm_const)]

use arduino_hal::Peripherals;

use atmega_hal::pac::CPU;
use atmega_hal::pac::TC1;
use avr_device::interrupt::CriticalSection;

use bootloader_rs::crc;
use bootloader_rs::run_program;
use bootloader_rs::spm;

use core::arch::asm;

use panic_halt as _;

use core::mem::MaybeUninit;

const RUNCOMMAND: u8 = 3;
const SPM_PAGESIZE: usize = 128;

#[repr(C)]
struct Frame {
    command: u8,
    page_address: u16,
    page: [u16; SPM_PAGESIZE / 2],
    checksum: u16,
}

// static RECEIVED_BYTE: Mutex<MyVolatileCell<Option<u8>>> = Mutex::new(MyVolatileCell{value: None});
// static STATE: Mutex<MyVolatileCell<CaptureState>> = Mutex::new(MyVolatileCell{value: CaptureState::Idle});

static RECEIVED_BYTE: MyVolatileCell<Option<u8>> = MyVolatileCell { value: None };
static STATE: MyVolatileCell<CaptureState> = MyVolatileCell {
    value: CaptureState::Idle,
};

// static gp0 : MyVolatileCell<*mut arduino_hal::pac::cpu::GPIOR0> = MyVolatileCell{value: (&() as *const ()) as *mut arduino_hal::pac::cpu::GPIOR0};

// static RECEIVED_WORD: Mutex<Cell<Option<u16>>> = Mutex::new(Cell::new(None));

/// # Safety
/// it's main, it can do what it wants
#[export_name = "main"]
pub unsafe extern "C" fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    init(&dp);
    let pgm_writer = spm::Writer::new(dp.CPU);

    // let pins = arduino_hal::pins!(dp);
    // let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    // ufmt::uwriteln!(&mut serial, "Start\r").void_unwrap();

    avr_device::interrupt::enable();

    loop {
        let mut buffer: MaybeUninit<Frame> = MaybeUninit::uninit();
        let mut crc: u16 = 0;
        let mut crc1: u16 = 0;
        let mut crc2: u16 = 0;

        for uninit_byte in buffer.as_bytes_mut() {
            let b = loop {
                if let Some(b) = avr_device::interrupt::free(|_| {
                    // let rb = RECEIVED_BYTE.borrow(cs);
                    // let b = rb.get();
                    // rb.set(None);
                    // b

                    RECEIVED_BYTE.take()
                }) {
                    break b;
                }
            };

            uninit_byte.write(b);

            crc2 = crc1;
            crc1 = crc;
            avr_device::interrupt::free(|_| {
                crc = crc::crc_xmodem_update_asm(crc, b);
            });
        }

        let frame: &Frame = buffer.assume_init_ref();

        avr_device::interrupt::free(|_| {
            // ufmt::uwriteln!(
            //     &mut serial,
            //     "{} {:#06x}, {:#06x}, {:#06x}",
            //     frame.command,
            //     frame.page_address,
            //     frame.checksum,
            //     crc2
            // ).void_unwrap();

            if frame.checksum == crc2 {
                if frame.command == RUNCOMMAND {
                    run_program();
                }

                pgm_writer.page_erase(frame.page_address);

                for (i, w) in frame.page.iter().enumerate() {
                    pgm_writer.page_fill(frame.page_address + (i as u16 * 2), *w);
                }

                pgm_writer.page_write(frame.page_address);
                pgm_writer.rww_enable();
            }
        });
    }
}

#[derive(Clone, Copy, PartialEq)]
enum CaptureState {
    Idle,
    Sync,
    Wait,
    Run,
}

#[export_name = "__vector_10"]
pub unsafe extern "avr-interrupt" fn TIMER1_CAPT() {
    const TIMER: *mut TC1 = (&() as *const ()) as *mut TC1; // cheeky

    static mut DELAY_TIME: u16 = 920;
    static mut BIT_COUNT: u8 = 0;

    let (state, time) = avr_device::interrupt::free(|cs| {
        let time = (*TIMER).icr1.read().bits();
        let state = STATE.get();

        (*TIMER)
            .tccr1b
            .modify(|r, w| w.ices1().bit(!r.ices1().bit()));
        (*TIMER).tcnt1.reset();
        //TODO: better accuracy by not resetting timer? - calc curr-prev time with overflow
        //Will break TIMER1_COMPA

        (state, time)
    });

    let new_state = match state {
        CaptureState::Idle => capt_idle(&mut BIT_COUNT),
        CaptureState::Sync => capt_sync(&mut BIT_COUNT),
        CaptureState::Wait => capt_wait(time, DELAY_TIME),
        CaptureState::Run => {
            capt_run(time, DELAY_TIME, &mut BIT_COUNT);
            CaptureState::Run
        }
    };

    if state == CaptureState::Sync {
        let new_delay_time = (DELAY_TIME + (time * 3 / 4)) / 2;
        DELAY_TIME = new_delay_time;
    }

    avr_device::interrupt::free(|cs| {
        STATE.set(new_state);
    });
}

#[export_name = "__vector_11"]
pub unsafe extern "avr-interrupt" fn TIMER1_COMPA() {
    without_interrupts(|cs| STATE.set(CaptureState::Idle));
}

unsafe fn capt_idle(bit_count: &mut u8) -> CaptureState {
    *bit_count = 0;
    CaptureState::Sync
}

unsafe fn capt_sync(bit_count: &mut u8) -> CaptureState {
    *bit_count += 1;

    if *bit_count >= 15 {
        *bit_count = 0;
        CaptureState::Wait
    } else {
        CaptureState::Sync
    }
}

unsafe fn capt_wait(time: u16, delay_time: u16) -> CaptureState {
    static mut CAP_COUNT: u8 = 0;

    if time < delay_time {
        if CAP_COUNT == 0 {
            CAP_COUNT = 1;
            CaptureState::Wait
        } else {
            CAP_COUNT = 0;
            CaptureState::Run
        }
    } else {
        CaptureState::Wait
    }
}

unsafe fn capt_run(time: u16, delay_time: u16, bit_count: &mut u8) {
    const CPU: *mut CPU = (&() as *const ()) as *mut CPU;
    let shift_reg = (*CPU).gpior0.as_ptr();

    if time < delay_time && (*shift_reg & 0x01) == 0x00 {
        *shift_reg |= 1;
    } else {
        *bit_count += 1;
        if *bit_count >= 8 {
            avr_device::interrupt::free(|cs| {
                // if RECEIVED_BYTE.borrow(cs).take().is_some() {
                //     panic!();
                // }
                RECEIVED_BYTE.set(Some(*shift_reg));
            });

            *bit_count = 0;
        }

        *shift_reg <<= 1;
    }
}

fn init(dp: &Peripherals) {
    // enable pullup on adc3
    dp.PORTC.portc.write(|w| w.pc3().set_bit());

    // switch off ADC so we can use the mux with the comparator
    dp.ADC.adcsra.write(|w| w.aden().clear_bit());

    // enable mux input to comparator
    dp.ADC.adcsrb.write(|w| w.acme().set_bit());

    // Switch mux to use A3 as the input
    dp.ADC.admux.write(|w| w.mux().adc3());

    // Disable digital input on AIN0 and AIN1
    dp.AC.didr1.write(|w| w.ain0d().set_bit().ain1d().set_bit());

    // setup timer1
    dp.TC1.tccr1a.reset();

    // no prescaler, enable noise cancelling, rising edge?
    dp.TC1
        .tccr1b
        .write(|w| w.cs1().direct().icnc1().set_bit().ices1().set_bit());

    //set interrupts
    dp.TC1
        .timsk1
        .write(|w| w.icie1().set_bit().ocie1a().set_bit());
    dp.TC1.ocr1a.write(|w| unsafe { w.bits(981 * 4) });

    dp.AC.acsr.write(|w| w.acic().set_bit());
}

fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce(&CriticalSection) -> R,
{
    unsafe {
        asm!("cli");
        let r = f(&CriticalSection::new());
        asm!("sei");
        r
    }
}

use core::ptr;

pub struct MyVolatileCell<T: ?Sized> {
    value: T,
}
unsafe impl<T: ?Sized> Sync for MyVolatileCell<T> {}

impl<T> MyVolatileCell<T> {
    pub const fn new(value: T) -> Self {
        MyVolatileCell { value: value }
    }

    /// Returns a copy of the contained value
    #[inline(always)]
    pub fn get(&self) -> T
    where
        T: Copy,
    {
        unsafe { ptr::read_volatile((&self.value) as *const T) }
    }

    /// Sets the contained value
    #[inline(always)]
    pub fn set(&self, value: T)
    where
        T: Copy,
    {
        unsafe { ptr::write_volatile((&self.value) as *const T as *mut T, value) }
    }

    pub fn replace(&self, val: T) -> T
    where
        T: Copy,
    {
        let old = self.get();
        self.set(val);
        old
    }
}

impl<T: Default> MyVolatileCell<T> {
    pub fn take(&self) -> T
    where
        T: Copy,
    {
        self.replace(Default::default())
    }
}
