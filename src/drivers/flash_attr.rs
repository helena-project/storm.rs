use core::prelude::*;
use hal::spi::*;
use hal::gpio::*;

pub struct FlashAttr<'a> {
    spi: &'a mut SPI,
    cs: &'a mut Pin,
    pub keys: [[u8,..8],..16]
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

fn get_key(spi: &SPI, cs: &Pin, idx : u8, key : &mut [u8,..8]) {
    let addr = idx as uint * 64;

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

impl <'a> FlashAttr<'a> {
    #[inline(never)]
    pub fn initialize(spi: &'a mut SPI, cs: &'a mut Pin,
                      mosi: &'a mut Pin, miso: &'a mut Pin,
                      sclk: &'a mut Pin) -> FlashAttr<'a> {
        let mut keys = [[0,..8],..16];

        cs.make_output();

        mosi.set_peripheral_function(PeripheralFunction::A);
        miso.set_peripheral_function(PeripheralFunction::A);
        sclk.set_peripheral_function(PeripheralFunction::A);

        spi.set_mode(Mode::Mode0);
        spi.set_baud_rate(8);

        for i in range(0,16) {
            get_key(spi, cs, i, &mut keys[i as uint]);
        }

        FlashAttr{spi: spi, cs: cs, keys: keys}
    }

    pub fn do_attr(&self, key : &str, f: |u8|) -> bool {
        let mut idx : uint = 0;
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

    pub fn do_attr_at_idx(&self, idx: uint, f: |u8|) {
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
            let _x : uint = _i;
            self.spi.write_read(0, false);
        }

        let len = self.spi.write_read(0, false) as uint;

        for _i in range(0, len) {
            f(self.spi.write_read(0, false) as u8);
        }

        sleep();
        self.cs.set();
    }

    pub fn get_attr(&self, key : &str, value: &mut [u8,..256]) -> uint {
        let mut len = -1;
        self.do_attr(key, |c| {
            len += 1;
            value[len] = c;
        });
        return len;
    }
}

fn cmp_keys(key1 : &str, key2: [u8,..8]) -> bool {
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

