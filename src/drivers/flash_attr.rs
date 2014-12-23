use core::slice::*;
use hal::spi::*;
use hal::gpio::*;

pub struct FlashAttr<'a> {
    spi: &'a mut SPI,
    cs: &'a mut Pin
}

fn sleep() {
    let mut i : uint = 0;
    loop {
        i += 1;
        if i > 1000 {
            return;
        }
    }
}

impl <'a> FlashAttr<'a> {
    pub fn initialize(spi: &'a mut SPI, cs: &'a mut Pin,
                      mosi: &'a mut Pin, miso: &'a mut Pin,
                      sclk: &'a mut Pin) -> FlashAttr<'a> {
        cs.make_output();

        mosi.set_peripheral_function(PeripheralFunction::A);
        miso.set_peripheral_function(PeripheralFunction::A);
        sclk.set_peripheral_function(PeripheralFunction::A);

        spi.set_mode(Mode::Mode0);
        spi.set_baud_rate(8);

        FlashAttr{spi: spi, cs: cs}
    }

    pub fn get_attr(&self, idx : u8, key : &mut [u8,..8]) {
        let addr = idx as uint * 64;

        self.cs.set();
        sleep();
        self.cs.clear();
        sleep();

        self.spi.write_read(0x1B, false);
        self.spi.write_read((addr >> 16 & 0xff) as u16, false);
        self.spi.write_read((addr >> 8 & 0xff) as u16, false);
        self.spi.write_read((addr & 0xff) as u16, false);
        self.spi.write_read(0, false);
        self.spi.write_read(0, false);

        for b in key.iter_mut() {
            *b = self.spi.write_read(0, false) as u8;
        }

        sleep();
        self.cs.set();
    }
}

