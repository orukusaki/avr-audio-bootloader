use atmega_hal::pac::SPI;
use atmega_hal::port::{mode::Output, Pin, PB2, PB3, PB5};
use atmega_hal::spi::SpiOps;
pub struct WriteOnlySpi {
    p: SPI,
    ss: Pin<Output, PB2>,
}

impl WriteOnlySpi {
    pub fn new(
        p: SPI,
        mut sck: Pin<Output, PB5>,
        mut _mosi: Pin<Output, PB3>,
        mut ss: Pin<Output, PB2>,
    ) -> WriteOnlySpi {
        p.spcr.write(|w| w.spe().set_bit().mstr().set_bit());
        ss.set_high();
        sck.set_high();

        WriteOnlySpi { p, ss }
    }

    pub fn transmit(&mut self, data: &[u8]) {
        self.ss.set_low();
        for b in data {
            self.send(*b);
        }
        self.ss.set_high();
    }

    fn send(&mut self, data: u8) {
        self.p.raw_write(data);
        loop {
            if self.p.raw_check_iflag() {
                break;
            }
        }
    }

    pub fn free(self) -> SPI {
        self.p
    }
}
