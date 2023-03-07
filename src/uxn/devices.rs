use super::Uxn;

pub trait Device {
    fn init(&mut self, uxn: &mut Uxn);
    fn cycle(&mut self, uxn: &mut Uxn);
    fn get(&mut self, port: u8);
    fn set_byte(&mut self, port: u8, value: u8);
    fn set_short(&mut self, port: u8, value: u16);
}
