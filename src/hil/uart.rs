pub trait UART {
    fn as_aurt(&self) { }
    fn send(&self, thing: &str);
}
