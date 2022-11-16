use crate::frame::ByteSource;
use atmega_hal::pac::{AC, ADC, TC0};
use atmega_hal::port::{
    mode::{Input, PullUp},
    Pin, PC3,
};

pub struct SignalReceiver {
    ac: AC,
    timer: TC0,
    delay_time: u8,
}

impl SignalReceiver {
    pub fn new(_pc3: Pin<Input<PullUp>, PC3>, adc: ADC, ac: AC, timer: TC0) -> SignalReceiver {
        Self::init(&adc, &ac, &timer);
        SignalReceiver {
            ac,
            timer,
            delay_time: 0,
        }
    }

    fn init(adc: &ADC, ac: &AC, timer: &TC0) {
        // switch off ADC so we can use the mux with the comparator
        adc.adcsra.write(|w| w.aden().clear_bit());

        // enable mux input to comparator
        adc.adcsrb.write(|w| w.acme().set_bit());

        // Switch mux to use A3 as the input
        adc.admux.write(|w| w.mux().adc3());

        // Disable digital input on AIN0 and AIN1
        ac.didr1.write(|w| w.ain0d().set_bit().ain1d().set_bit());

        // setup timer0
        timer.tccr0b.write(|w| w.cs0().prescale_8())
    }

    fn ac_value(&self) -> bool {
        self.ac.acsr.read().aco().bit_is_set()
    }

    fn wait_for_edge(&self, ac_state: bool) {
        while self.ac_value() == ac_state {}
    }

    fn wait_for_time(&self) {
        self.timer.tcnt0.reset();
        while self.timer.tcnt0.read().bits() < self.delay_time {}
    }
}

impl ByteSource for SignalReceiver {
    fn get(&self) -> u8 {
        let mut ac_state = self.ac_value();
        let mut b: u8 = 0;
        for _ in 0..8 {
            self.wait_for_edge(ac_state);
            ac_state = !ac_state;

            self.wait_for_time();
            let new_ac_state = self.ac_value();

            b <<= 1;
            b |= (new_ac_state ^ ac_state) as u8;
            ac_state = new_ac_state;
        }
        b
    }

    fn sync(&mut self) {
        let mut total_time: u16 = 0;
        let mut ac_state: bool = self.ac_value();

        for i in 0..16 {
            self.timer.tcnt0.reset();
            self.wait_for_edge(ac_state);
            ac_state = !ac_state;

            if i >= 8 {
                total_time += self.timer.tcnt0.read().bits() as u16;
            }
        }

        self.delay_time = (total_time * 3 / 4 / 8) as u8;
        // wait for start (1) bit
        loop {
            self.wait_for_edge(ac_state);
            ac_state = !ac_state;
            self.wait_for_time();
            if self.ac_value() != ac_state {
                break;
            }
        }
    }
}
