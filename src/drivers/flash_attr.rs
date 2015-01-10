use core::prelude::*;
use hil::gpio;
use hil::gpio::PeripheralFunction;
use hil::spi;
use hil::spi::Mode::*;

pub struct FlashAttr<SPI : spi::SPI, Pin : gpio::Pin> {
    spi: SPI,
    cs: Pin,
    keys: [[u8;8];16]
}

fn sleep() {
    let mut i : usize = 0;
    loop {
        i += 1;
        if i > 1000 {
            return;
        }
    }
}

fn get_key<SPI : spi::SPI, Pin : gpio::Pin>
        (spi: &SPI, cs: &Pin, idx : u8, key : &mut [u8;8]) {
    let addr = idx as usize * 64;

    cs.set();
    sleep();
    cs.clear();
    sleep();

    spi.write_read(0x1B, false);
    spi.write_read((addr >> 16 & 0xff) as u16, false);
    spi.write_read((addr >> 8 & 0xff) as u16, false);
    spi.write_read((addr & 0xff) as u16, false);
    spi.write_read(0, false);
    spi.write_read(0, false);

    for b in key.iter_mut() {
        *b = spi.write_read(0, false) as u8;
    }

    sleep();
    cs.set();
}

impl <SPI : spi::SPI, Pin : gpio::Pin> FlashAttr<SPI, Pin> {
    #[inline(never)]
    pub fn initialize(spi: SPI, cs: Pin,
                      mosi: Pin, miso: Pin,
                      sclk: Pin) -> FlashAttr<SPI, Pin> {
        let mut keys = [[0;8];16];

        cs.make_output();

        mosi.set_peripheral_function(PeripheralFunction::A);
        miso.set_peripheral_function(PeripheralFunction::A);
        sclk.set_peripheral_function(PeripheralFunction::A);

        spi.set_mode(Mode0);
        spi.set_baud_rate(8);

        for i in range(0,16) {
            get_key(&spi, &cs, i, &mut keys[i as usize]);
        }

        FlashAttr{spi: spi, cs: cs, keys: keys}
    }

    pub fn do_attr<F: FnMut(u8)>(self, key : &str, f: F) -> bool {
        let mut idx : usize = 0;
        let mut res_idx = None;
        for k in self.keys.iter() {
            if cmp_keys(key, *k) {
                res_idx = Some(idx);
                break;
            }
            idx += 1;
        }

        match res_idx {
            None => false,
            Some(ridx) => {
                self.do_attr_at_idx(ridx, f);
                true
            }
        }
    }

    pub fn do_attr_at_idx<F: FnMut(u8)>(self, idx: usize, mut f: F) {
        let addr = idx * 64;

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

        for _i in range(0,8) {
            let _x : usize = _i;
            self.spi.write_read(0, false);
        }

        let len = self.spi.write_read(0, false) as usize;

        for _i in range(0, len) {
            f(self.spi.write_read(0, false) as u8);
        }

        sleep();
        self.cs.set();
    }

    pub fn get_attr(self, key : &str, value: &mut [u8;256]) -> usize {
        let mut len = -1;
        self.do_attr(key, |&mut: c| {
            len += 1;
            value[len] = c;
        });
        return len;
    }
}

fn cmp_keys(key1 : &str, key2: [u8;8]) -> bool {
    use core::prelude::*;

    if key1.len() > 8 {
        return false;
    }

    let mut idx = 0;
    for b in key1.bytes() {
        if b != key2[idx] {
            return false;
        }
        idx += 1;
    }
    if idx < 8 {
        return key2[idx] == 0;
    }
    return true;
}

