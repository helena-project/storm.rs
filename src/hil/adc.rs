pub trait ADCMaster {
    fn enable(&mut self) -> bool;
    fn disable(&mut self);
    fn is_enabled(&self) -> bool;
    fn sample(&mut self) -> u16;
}

