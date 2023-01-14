struct Stack {
    data: Vec<u8>,
}

impl Stack {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn push_byte(&mut self, byte: u8) {
        self.data.push(byte)
    }

    fn pop_byte(&mut self) -> u8 {
        self.data.pop().unwrap()
    }

    fn push_short(&mut self, short: u16) {
        let bytes = short.to_be_bytes();
        self.data.extend_from_slice(&bytes);
    }

    fn pop_short(&mut self) -> u16 {
        let lower = self.data.pop().unwrap();
        let upper = self.data.pop().unwrap();

        return (upper << 8) + lower;
    }
}
