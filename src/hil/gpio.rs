
#[deriving(Copy)]
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

pub trait Pin {
    fn make_output(&self);
    fn set(&self);
    fn clear(&self);
    fn toggle(&self);
    fn set_peripheral_function(&self, PeripheralFunction);
}

