// TODO: This should be in HIL.

pub trait UART {
    fn as_aurt(&self) { }
    fn send(&self, thing: &str);
}
