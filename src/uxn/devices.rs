use super::Uxn;

pub trait Device {
    fn init(&mut self, uxn: &mut Uxn);
    fn cycle(&mut self, uxn: &mut Uxn);
    fn get(&mut self, port: u8) -> u8;
    fn set_byte(&mut self, port: u8, value: u8);
    fn set_short(&mut self, port: u8, value: u16);
}

pub struct Console {
    mem: [u8; 16],
}

impl Console {
    pub fn new() -> Self {
        Self { mem: [0; 16] }
    }

    fn write(&mut self) {
        let byte = self.mem[0x8] as char;
        print!("{byte}");
    }
}

impl Device for Console {
    fn init(&mut self, _uxn: &mut Uxn) {}
    fn cycle(&mut self, _uxn: &mut Uxn) {}
    fn get(&mut self, port: u8) -> u8 {
        self.mem[port as usize]
    }
    fn set_byte(&mut self, port: u8, value: u8) {
        self.mem[port as usize] = value;
        match port {
            0x8 => self.write(),
            _ => (),
        }
    }
    fn set_short(&mut self, _port: u8, _value: u16) {
        todo!()
    }
}
