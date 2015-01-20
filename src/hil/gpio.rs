use core::prelude::*;

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

impl Copy for PeripheralFunction {}

pub trait Pin {
    // What is make output?
    fn make_output(&mut self);
    fn set(&mut self);
    fn clear(&mut self);
    fn toggle(&mut self);
    fn select_peripheral(&mut self, PeripheralFunction);
}

