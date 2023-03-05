struct Stack {
    data: Vec<u8>,
    keep_mode: bool,
    pop_offset: usize,
}

impl Stack {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            keep_mode: false,
            pop_offset: 0,
        }
    }

    fn set_keep_mode(&mut self, mode: bool) {
        self.pop_offset = 0;
        self.keep_mode = mode;
    }

    fn push_byte(&mut self, byte: u8) {
        self.data.push(byte);
        self.pop_offset += 1;
    }

    fn pop_byte(&mut self) -> u8 {
        if self.keep_mode {
            let value = self.data[self.data.len() - self.pop_offset - 1];
            self.pop_offset += 1;
            value
        } else {
            self.data.pop().unwrap()
        }
    }

    fn push_short(&mut self, short: u16) {
        self.push_byte((short >> 8) as u8);
        self.push_byte(short as u8);
    }

    fn pop_short(&mut self) -> u16 {
        let lower = self.pop_byte();
        let upper = self.pop_byte();

        return ((upper as u16) << 8) + lower as u16;
    }
}

#[repr(u8)]
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

    fn eval_vector(&mut self) {
        loop {
            let instr = self.mem[self.pc as usize];

            let (wst, rst) = (&mut self.wst, &mut self.rst);
            // Working and return stacks are swapped in return mode
            if instr & 0x40 != 0 {
                std::mem::swap(wst, rst);
            }

            // Activate keep mode
            if instr & 0x80 != 0 {
                wst.set_keep_mode(true);
            }

            let short_mode = instr & 0x20 != 0;

            macro_rules! pop {
                ($stack:expr) => {
                    if short_mode {
                        $stack.pop_short()
                    } else {
                        $stack.pop_byte() as u16
                    }
                };
            }

            macro_rules! push {
                ($stack:expr, $value:expr) => {
                    if short_mode {
                        $stack.push_short($value)
                    } else {
                        $stack.push_byte($value as u8)
                    }
                };
            }

            use Instruction::*;
            match unsafe { std::mem::transmute(instr) } {
                BRK => return,
                INC => {
                    let a = pop!(wst);
                    push!(wst, a + 1);
                }
                POP => {
                    pop!(wst);
                }
                NIP => {
                    let a = pop!(wst);
                    pop!(wst);
                    push!(wst, a);
                }
                SWP => {
                    let a = pop!(wst);
                    let b = pop!(wst);
                    push!(wst, a);
                    push!(wst, b);
                }
                ROT => {
                    let a = pop!(wst);
                    let b = pop!(wst);
                    let c = pop!(wst);
                    push!(wst, b);
                    push!(wst, a);
                    push!(wst, c);
                }
                DUP => {
                    let a = pop!(wst);
                    push!(wst, a);
                    push!(wst, a);
                }
                OVR => {
                    let a = pop!(wst);
                    let b = pop!(wst);
                    push!(wst, b);
                    push!(wst, a);
                    push!(wst, b);
                }
                EQU => {
                    let b = pop!(wst);
                    let a = pop!(wst);
                    push!(wst, (a == b) as u16);
                }
                NEQ => {
                    let b = pop!(wst);
                    let a = pop!(wst);
                    push!(wst, (a != b) as u16);
                }
                GTH => {
                    let b = pop!(wst);
                    let a = pop!(wst);
                    push!(wst, (a > b) as u16);
                }
                LTH => {
                    let b = pop!(wst);
                    let a = pop!(wst);
                    push!(wst, (a < b) as u16)
                }
                _ => todo!(),
            }
        }
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

    // Test keep mode
    s.push_byte(0x12);
    s.push_byte(0x34);
    s.set_keep_mode(true);
    s.push_byte(0x56);
    assert_eq!(s.pop_byte(), 0x34);
    assert_eq!(s.pop_byte(), 0x12);
    s.set_keep_mode(false);
    assert_eq!(s.pop_byte(), 0x56);
    assert_eq!(s.pop_short(), 0x1234);
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
