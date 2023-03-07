mod devices;
mod stack;

pub use devices::Device;
pub use stack::Stack;

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

pub struct Uxn<'a> {
    /// Memory: 64 kB
    pub mem: [u8; 0x10000],
    /// Program Counter
    pc: u16,
    /// Working Stack
    wst: Stack,
    /// Return Stack
    rst: Stack,
    devices: [Option<&'a dyn Device>; 16],
}

impl<'a> Uxn<'a> {
    fn new() -> Self {
        Self {
            mem: [0; 0x10000],
            pc: 0x0100,
            wst: Stack::new(),
            rst: Stack::new(),
            devices: [None; 16],
        }
    }

    fn mount_device(&mut self, device: &'a dyn Device, port: u8) {
        match self.devices[port as usize] {
            Some(_) => panic!("Another device already mounted on port"),
            None => self.devices[port as usize] = Some(device),
        }
    }

    fn load_rom(&mut self, rom: &[u8]) {
        let start = 0x0100;
        let end = 0x0100 + rom.len();

        self.mem[start..end].copy_from_slice(rom);
        self.pc = 0x0100;
    }

    fn eval_vector(&mut self, addr: u16) {
        self.pc = addr;

        loop {
            let instr = self.mem[self.pc as usize];

            println!("{:#06x}, {instr:#04x}", self.pc);
            println!("{:?}", self.wst.data);

            self.pc += 1;

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

            macro_rules! jump {
                ($addr:expr) => {
                    if short_mode {
                        self.pc = $addr
                    } else {
                        self.pc += $addr
                    }
                };
            }

            macro_rules! peek {
                ($addr:expr) => {
                    if short_mode {
                        let high = self.mem[$addr as usize];
                        let low = self.mem[$addr as usize + 1];
                        u16::from_be_bytes([high, low])
                    } else {
                        self.mem[$addr as usize] as u16
                    }
                };
            }

            macro_rules! poke {
                ($addr:expr, $value:expr) => {
                    if short_mode {
                        let high = ($value >> 8) as u8;
                        let low = $value as u8;
                        self.mem[$addr as usize] = high;
                        self.mem[$addr as usize + 1] = low;
                    } else {
                        self.mem[$addr as usize] = $value as u8;
                    }
                };
            }

            use Instruction::*;
            match unsafe { std::mem::transmute(instr & 0b00011111) } {
                BRK => match instr >> 5 {
                    0 => return,
                    1 => {
                        let cond = pop!(wst);
                        if cond != 0 {
                            self.pc += u16::from_be_bytes([
                                self.mem[self.pc as usize],
                                self.mem[self.pc as usize + 1],
                            ]);
                        }
                        self.pc += 2
                    }
                    2 => {
                        let addr = u16::from_be_bytes([
                            self.mem[self.pc as usize],
                            self.mem[self.pc as usize + 1],
                        ]);
                        self.pc += addr + 2;
                    }
                    3 => {
                        rst.push_short(self.pc + 2);
                        let addr = u16::from_be_bytes([
                            self.mem[self.pc as usize],
                            self.mem[self.pc as usize + 1],
                        ]);
                        self.pc += addr + 2;
                    }
                    4 | 5 | 6 | 7 => {
                        let value = peek!(self.pc);
                        self.pc += if short_mode { 2 } else { 1 };
                        push!(wst, value);
                    }
                    _ => unreachable!(),
                },
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
                JMP => {
                    let addr = pop!(wst);
                    jump!(addr)
                }
                JCN => {
                    let addr = pop!(wst);
                    let cond = wst.pop_byte();

                    if cond != 0 {
                        jump!(addr)
                    }
                }
                JSR => {
                    let addr = pop!(wst);
                    rst.push_short(self.pc);
                    jump!(addr)
                }
                STH => {
                    let a = pop!(wst);
                    push!(rst, a);
                }
                LDZ => {
                    let addr = wst.pop_byte();
                    let value = peek!(addr);
                    push!(wst, value);
                }
                STZ => {
                    let addr = wst.pop_byte();
                    let value = pop!(wst);
                    poke!(addr, value);
                }
                LDR => {
                    let offset: i8 = unsafe { std::mem::transmute(wst.pop_byte()) };
                    let addr = self.pc.wrapping_add_signed(offset as i16);
                    let value = peek!(addr);
                    push!(wst, value);
                }
                STR => {
                    let offset: i8 = unsafe { std::mem::transmute(wst.pop_byte()) };
                    let addr = self.pc.wrapping_add_signed(offset as i16);
                    let value = pop!(wst);
                    poke!(addr, value);
                }
                LDA => {
                    let addr = wst.pop_short();
                    let value = peek!(addr);
                    push!(wst, value);
                }
                STA => {
                    let addr = wst.pop_short();
                    let value = pop!(wst);
                    poke!(addr, value)
                }
                DEI => {
                    todo!();
                }
                DEO => {
                    todo!();
                }
                ADD => {
                    let b = pop!(wst);
                    let a = pop!(wst);
                    push!(wst, a + b);
                }
                SUB => {
                    let b = pop!(wst);
                    let a = pop!(wst);
                    push!(wst, a - b);
                }
                MUL => {
                    let b = pop!(wst);
                    let a = pop!(wst);
                    push!(wst, a * b);
                }
                DIV => {
                    let b = pop!(wst);
                    let a = pop!(wst);
                    push!(wst, a / b);
                }
                AND => {
                    let b = pop!(wst);
                    let a = pop!(wst);
                    push!(wst, a & b);
                }
                ORA => {
                    let b = pop!(wst);
                    let a = pop!(wst);
                    push!(wst, a | b);
                }
                EOR => {
                    let b = pop!(wst);
                    let a = pop!(wst);
                    push!(wst, a ^ b);
                }
                SFT => {
                    let a = pop!(wst);
                    let shift = wst.pop_byte();

                    let right = shift & 0xf;
                    let left = shift >> 4;

                    let result = if short_mode {
                        (a >> right) << left
                    } else {
                        ((a as u8 >> right) << left) as u16
                    };
                    push!(wst, result)
                }
            }
            wst.set_keep_mode(false);
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
    let mut uxn = Uxn::new();
    let rom: [u8; 4] = [0x12, 0x34, 0x56, 0x78];

    // Verify that first four bytes are the ROM bytes
    uxn.load_rom(&rom);
    assert_eq!(uxn.mem[0x0100..0x0104], [0x12, 0x34, 0x56, 0x78]);

    // Verify that the rest of the memory is zeroed
    for byte in uxn.mem[0x0104..].iter() {
        assert_eq!(*byte, 0_u8);
    }
}

#[test]
pub fn test_cpu_opcodes() {
    macro_rules! stack_assert {
        ($program:expr, $stack:expr) => {
            let mut uxn = Uxn::new();
            uxn.load_rom($program);
            uxn.eval_vector(0x0100);
            let stack = &uxn.wst.data;
            assert_eq!(stack.as_slice(), $stack);
        };
    }

    // LIT 12 ( 12 )
    stack_assert!(&[0x80, 0x12], [0x12]);
    // LIT2 1234 ADD ( 46 )
    stack_assert!(&[0xa0, 0x12, 0x34, 0x18], [0x46]);
    // LIT 10 DUP ( 10 10 )
    stack_assert!(&[0x80, 0x10, 0x06], [0x10, 0x10]);
    // LIT2 1234 SWP ( 34 12 )
    stack_assert!(&[0xa0, 0x12, 0x34, 0x04], [0x34, 0x12]);
    // LIT2 1234 ADDk ( 12 34 46 )
    stack_assert!(&[0xa0, 0x12, 0x34, 0x98], [0x12, 0x34, 0x46]);
    // LIT 02 JMP LIT 12 LIT 34 ( 34 )
    stack_assert!(&[0x80, 0x02, 0x0c, 0x80, 0x12, 0x80, 0x34], [0x34]);
}
