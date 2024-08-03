use core::fmt;
use std::fmt::Write;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Reg8 {
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101,
    F = 0b110,
    A = 0b111,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Reg16 {
    BC = 0b00,
    DE = 0b01,
    HL = 0b10,
    SP = 0b11,
}

pub struct Cpu {
    // Registers
    pub general_purpose_regs: [u8; 16],
    pub program_counter: u16,
    pub stack_pointer: u16,
    pub index_x: u16,
    pub index_y: u16,
    pub memory_refresh: u8,
    // Memory
    pub memory: [u8; 0x10000],
    pub jump_address: Option<u16>,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            general_purpose_regs: Default::default(),
            program_counter: Default::default(),
            stack_pointer: Default::default(),
            index_x: Default::default(),
            index_y: Default::default(),
            memory_refresh: Default::default(),
            memory: [0; 0x10000],
            jump_address: None,
        }
    }
}

impl fmt::Display for Reg8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{:?}", self).to_lowercase())
    }
}
impl fmt::Display for Reg16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{:?}", self).to_lowercase())
    }
}
impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let strings = self
            .general_purpose_regs
            .iter()
            .take(8)
            .enumerate()
            .map(|(reg, val)| format!("{}=0x{:02x} ", Reg8::from(reg as u8), val))
            .collect::<Vec<String>>();

        for r in strings {
            f.write_str(&r)?;
        }
        f.write_str(&format!("pc: 0x{:04x} ", self.program_counter))?;
        f.write_str(&format!("sp: 0x{:04x} ", self.stack_pointer))?;
        f.write_str(&format!("index_x: 0x{:04x} ", self.index_x))?;
        f.write_str(&format!("index_y: 0x{:04x} ", self.index_y))?;

        Ok(())
    }
}

impl From<u8> for Reg8 {
    fn from(val: u8) -> Self {
        assert_eq!(val & 0b111, val);
        unsafe { std::mem::transmute(val) }
    }
}

impl From<u8> for Reg16 {
    fn from(val: u8) -> Self {
        assert_eq!(val & 0b11, val);
        unsafe { std::mem::transmute(val) }
    }
}

impl Cpu {
    pub fn read_reg8(&self, reg: Reg8) -> u8 {
        self.general_purpose_regs[reg as usize]
    }

    pub fn read_reg16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::BC => u16::from_be_bytes([self.read_reg8(Reg8::B), self.read_reg8(Reg8::C)]),
            Reg16::DE => u16::from_be_bytes([self.read_reg8(Reg8::D), self.read_reg8(Reg8::E)]),
            Reg16::HL => self.read_mem16(u16::from_be_bytes([
                self.read_reg8(Reg8::H),
                self.read_reg8(Reg8::L),
            ])),
            Reg16::SP => self.stack_pointer,
        }
    }

    pub fn write_reg16(&mut self, reg: Reg16, value: u16) {
        let [high, low] = value.to_be_bytes();
        match reg {
            Reg16::BC => {
                self.write_reg8(Reg8::B, high);
                self.write_reg8(Reg8::C, low);
            }
            Reg16::DE => {
                self.write_reg8(Reg8::D, high);
                self.write_reg8(Reg8::E, low);
            }
            Reg16::HL => {
                let address = self.read_reg16(Reg16::HL);
                self.write_mem16(address, value);
            }
            Reg16::SP => {
                self.stack_pointer = value;
            }
        }
    }

    pub fn read_mem8(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn read_mem16(&self, address: u16) -> u16 {
        u16::from_be_bytes([self.read_mem8(address), self.read_mem8(address + 1)])
    }

    pub fn write_mem8(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    pub fn write_mem16(&mut self, address: u16, value: u16) {
        let [high, low] = value.to_be_bytes();
        self.write_mem8(address, high);
        self.write_mem8(address + 1, low);
    }

    pub fn write_reg8(&mut self, reg: Reg8, value: u8) {
        self.general_purpose_regs[reg as usize] = value;
    }

    pub fn load_mem8_const(&mut self) -> u8 {
        let value = self.memory[self.program_counter as usize];
        self.program_counter = self.program_counter.checked_add(1).unwrap();
        value
    }

    pub fn load_mem16_const(&mut self) -> u16 {
        u16::from_be_bytes([self.load_mem8_const(), self.load_mem8_const()])
    }
}
