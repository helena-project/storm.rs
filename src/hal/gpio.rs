use core::intrinsics;

#[repr(C, packed)]
struct GpioPort {
    gper : u32,
    gpers : u32,
    gperc : u32,
    gpert : u32,
    pmr0 : u32,
    pmr0s : u32,
    pmr0c : u32,
    pmr0t : u32,
    //0x20
    pmr1 : u32,
    pmr1s : u32,
    pmr1c : u32,
    pmr1t : u32,
    pmr2 : u32,
    pmr2s : u32,
    pmr2c : u32,
    pmr2t : u32,
    //0x40
    oder : u32,
    oders : u32,
    oderc : u32,
    odert : u32,
    ovr : u32,
    ovrs : u32,
    ovrc : u32,
    ovrt : u32,
    //0x60
    pvr : u32,
    reserved0 : [u32, ..3],
    puer : u32,
    puers : u32,
    puerc : u32,
    puert : u32,
    //0x80
    pder : u32,
    pders : u32,
    pderc : u32,
    pdert : u32,
    ier : u32,
    iers : u32,
    ierc : u32,
    iert : u32,
    //0xA0
    imr0 : u32,
    imr0s : u32,
    imr0c : u32,
    imr0t : u32,
    imr1 : u32,
    imr1s : u32,
    imr1c : u32,
    imr1t : u32,
    //0xC0
    gfer : u32,
    gfers : u32,
    gferc : u32,
    gfert : u32,
    ifr : u32,
    reserved1 : u32,
    ifrc : u32,
    reserved2 : u32,
    //0xE0
    reserved3 : [u32, ..8],
    //0x100
    odcr0 : u32,
    odcr0s : u32,
    odcr0c : u32,
    odcr0t : u32,
    odcr1 : u32,
    odcr1s : u32,
    odcr1c : u32,
    odcr1t : u32,
    //0x120
    reserved4 : [u32, ..4],
    osrr0 : u32,
    osrr0s : u32,
    osrr0c : u32,
    osrr0t : u32,
    //0x140
    reserved5: [u32, ..8],
    //0x160
    ster : u32,
    sters : u32,
    sterc : u32,
    stert : u32,
    reserved6 : [u32, ..4],
    //0x180
    ever : u32,
    evers : u32,
    everc : u32,
    evert : u32,
    reserved7: [u32, ..112]
    //0x200 end
}

pub enum Port {
    PORT0 = 0x400E1000,
    PORT1 = 0x400E1200,
    PORT2 = 0x400E1400
}

pub enum PeripheralFunction {
    A = 0b000,
    B = 0b001,
    C = 0b010,
    D = 0b011,
    E = 0b100,
    F = 0b101,
    G = 0b110,
    H = 0b111
}

macro_rules! gpio_port(
    ($addr : expr) => (
        unsafe {
            &mut *($addr as u32 as *mut GpioPort)
        }
    );
)

pub struct Pin {
    pub bus: Port,
    pub pin: uint,
}

impl Pin {
    pub fn make_output(&self) {
        let gpio = gpio_port!(self.bus);
        let p = 1 << self.pin;
        unsafe {
            intrinsics::volatile_store(&mut gpio.gpers, p);
            intrinsics::volatile_store(&mut gpio.oders, p);
            intrinsics::volatile_store(&mut gpio.sterc, p);
        }
    }

    pub fn set_peripheral_function(&self, peripheral : PeripheralFunction) {
        let gpio = gpio_port!(self.bus);
        let p = 1 << self.pin;
        unsafe {
            // clear GPIO enable for pin
            intrinsics::volatile_store(&mut gpio.gperc, p);

            // Set PMR0-2 according to passed in peripheral
            let p = 1 << self.pin as uint;
            let periph = peripheral as uint;

            intrinsics::volatile_store(&mut gpio.pmr0,
              ((periph & 1) << p) as u32); // First bit of peripheral
            intrinsics::volatile_store(&mut gpio.pmr1,
              ((periph & 2 >> 1) << p) as u32); // Second bit of peripheral
            intrinsics::volatile_store(&mut gpio.pmr2,
             ((periph & 4 >> 2) << p) as u32); // Third bit of peripheral
        }
    }

    pub fn toggle(&self) {
        let gpio = gpio_port!(self.bus);
        let p = 1 << self.pin;
        unsafe {
            intrinsics::volatile_store(&mut gpio.ovrt, p);
        }
    }

    pub fn set(&self) {
        let gpio = gpio_port!(self.bus);
        let p = 1 << self.pin;
        unsafe {
            intrinsics::volatile_store(&mut gpio.ovrs, p);
        }
    }

    pub fn clear(&self) {
        let gpio = gpio_port!(self.bus);
        let p = 1 << self.pin;
        unsafe {
            intrinsics::volatile_store(&mut gpio.ovrc, p);
        }
    }
}

