pub struct Stack {
    pub data: Vec<u8>,
    keep_mode: bool,
    pop_offset: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            keep_mode: false,
            pop_offset: 0,
        }
    }

    pub fn set_keep_mode(&mut self, mode: bool) {
        self.pop_offset = 0;
        self.keep_mode = mode;
    }

    pub fn push_byte(&mut self, byte: u8) {
        self.data.push(byte);
        self.pop_offset += 1;
    }

    pub fn pop_byte(&mut self) -> u8 {
        if self.data.len() == 0 {
            panic!("Stack underflow");
        }

        if self.keep_mode {
            let value = self.data[self.data.len() - self.pop_offset - 1];
            self.pop_offset += 1;
            value
        } else {
            self.data.pop().unwrap()
        }
    }

    pub fn push_short(&mut self, short: u16) {
        self.push_byte((short >> 8) as u8);
        self.push_byte(short as u8);
    }

    pub fn pop_short(&mut self) -> u16 {
        let lower = self.pop_byte();
        let upper = self.pop_byte();

        return ((upper as u16) << 8) + lower as u16;
    }
}
