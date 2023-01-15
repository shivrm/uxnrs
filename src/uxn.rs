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

        return ((upper as u16) << 8) + lower as u16;
    }
}

enum Instruction {
    BRK = 0x00, // Also represents JCI, JMI, JSI, LIT, LIT2, LITr, LIT2r
    INC = 0x01,
    POP = 0x02,
    NIP = 0x03,
    SWP = 0x04,
    ROT = 0x05,
    DUP = 0x06,
    OVR = 0x07,
    EQU = 0x08,
    NEQ = 0x09,
    GTH = 0x0a,
    LTH = 0x0b,
    JMP = 0x0c,
    JCN = 0x0d,
    JSR = 0x0e,
    STH = 0x0f,
    LDZ = 0x10,
    STZ = 0x11,
    LDR = 0x12,
    STR = 0x13,
    LDA = 0x14,
    STA = 0x15,
    DEI = 0x16,
    DEO = 0x17,
    ADD = 0x18,
    SUB = 0x19,
    MUL = 0x1a,
    DIV = 0x1b,
    AND = 0x1c,
    ORA = 0x1d,
    EOR = 0x1e,
    SFT = 0x1f,
}

struct Cpu {
    /// Memory: 64 kB
    mem: [u8; 0x10000],
    /// Program Counter
    pc: u16,
    /// Working Stack
    wst: Stack,
    /// Return Stack
    rst: Stack,
}

impl Cpu {
    fn new() -> Self {
        Self {
            mem: [0; 0x10000],
            pc: 0x0100,
            wst: Stack::new(),
            rst: Stack::new(),
        }
    }

    fn load_rom(&mut self, rom: &[u8]) {
        let start = 0x0100;
        let end = 0x0100 + rom.len();

        self.mem[start..end].copy_from_slice(rom);
        self.pc = 0x0100;
    }
}

#[test]
fn test_stack() {
    let mut s = Stack::new();

    // Test byte pushing and popping
    s.push_byte(0x10);
    s.push_byte(0x20);
    assert_eq!(s.pop_byte(), 0x20);
    assert_eq!(s.pop_byte(), 0x10);

    // Test short pushing and popping
    s.push_short(0x1234);
    s.push_short(0x5678);
    assert_eq!(s.pop_short(), 0x5678);
    assert_eq!(s.pop_short(), 0x1234);

    // Test conversion of shorts into bytes
    s.push_short(0x1234);
    assert_eq!(s.pop_byte(), 0x34);
    assert_eq!(s.pop_byte(), 0x12);

    // Test conversion of bytes into shorts
    s.push_byte(0x56);
    s.push_byte(0x78);
    assert_eq!(s.pop_short(), 0x5678);
}

#[test]
fn test_load_rom() {
    let mut cpu = Cpu::new();
    let rom: [u8; 4] = [0x12, 0x34, 0x56, 0x78];

    // Verify that first four bytes are the ROM bytes
    cpu.load_rom(&rom);
    assert_eq!(cpu.mem[0x0100..0x0104], [0x12, 0x34, 0x56, 0x78]);

    // Verify that the rest of the memory is zeroed
    for byte in cpu.mem[0x0104..].iter() {
        assert_eq!(*byte, 0_u8);
    }
}
