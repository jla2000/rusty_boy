use bitmatch::bitmatch;
use std::{collections::HashMap, fmt::format};

struct Context {
    // Registers
    general_purpose_regs: [u8; 16],
    program_counter: u16,
    stack_pointer: u16,
    index_x: u16,
    index_y: u16,
    memory_refresh: u8,
    // Memory
    memory: [u8; 0xffff],
}

impl Default for Context {
    fn default() -> Self {
        Self {
            general_purpose_regs: Default::default(),
            program_counter: Default::default(),
            stack_pointer: Default::default(),
            index_x: Default::default(),
            index_y: Default::default(),
            memory_refresh: Default::default(),
            memory: [0; 0xffff],
        }
    }
}

const REG_B: u8 = 0b000;
const REG_C: u8 = 0b001;
const REG_D: u8 = 0b010;
const REG_E: u8 = 0b011;
const REG_H: u8 = 0b100;
const REG_L: u8 = 0b101;
const REG_F: u8 = 0b110;
const REG_A: u8 = 0b111;
const REG_BC: u8 = 0b00;
const REG_DE: u8 = 0b01;
const REG_HL: u8 = 0b10;
const REG_SP: u8 = 0b11;

impl Context {
    fn read_reg8(&self, reg: u8) -> u8 {
        self.general_purpose_regs[reg as usize]
    }

    fn read_reg16(&self, reg: u8) -> u16 {
        match reg {
            REG_BC => u16::from_be_bytes([self.read_reg8(REG_B), self.read_reg8(REG_C)]),
            REG_DE => u16::from_be_bytes([self.read_reg8(REG_D), self.read_reg8(REG_E)]),
            REG_HL => self.read_u16(u16::from_be_bytes([
                self.read_reg8(REG_H),
                self.read_reg8(REG_L),
            ])),
            REG_SP => self.stack_pointer,
            _ => panic!("Unknown register"),
        }
    }

    fn write_reg16(&mut self, reg: u8, value: u16) {
        let [high, low] = value.to_be_bytes();
        match reg {
            REG_BC => {
                self.write_reg8(REG_B, high);
                self.write_reg8(REG_C, low);
            }
            REG_DE => {
                self.write_reg8(REG_D, high);
                self.write_reg8(REG_E, low);
            }
            REG_HL => {
                let address = self.read_reg16(REG_HL);
                self.write_u16(address, value);
            }
            REG_SP => {
                self.stack_pointer = value;
            }
            _ => panic!("Unknown register"),
        }
    }

    fn read_u8(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn read_u16(&self, address: u16) -> u16 {
        u16::from_be_bytes([self.read_u8(address), self.read_u8(address + 1)])
    }

    fn write_u8(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    fn write_u16(&mut self, address: u16, value: u16) {
        let [high, low] = value.to_be_bytes();
        self.write_u8(address, high);
        self.write_u8(address + 1, low);
    }

    fn write_reg8(&mut self, reg: u8, value: u8) {
        self.general_purpose_regs[reg as usize] = value;
    }

    fn load_u8_const(&mut self) -> u8 {
        self.program_counter += 1;
        self.memory[self.program_counter as usize]
    }

    fn load_u16_const(&mut self) -> u16 {
        u16::from_be_bytes([self.load_u8_const(), self.load_u8_const()])
    }
}

struct Instruction {
    pub execute: Box<dyn Fn(&mut Context)>,
    pub disassembly: String,
}

fn reg_to_str(reg: u8) -> &'static str {
    match reg {
        REG_B => "b",
        REG_C => "c",
        REG_D => "d",
        REG_E => "e",
        REG_H => "h",
        REG_L => "l",
        REG_F => "f",
        REG_A => "a",
        _ => "<invalid>",
    }
}

fn load_reg8(dst: u8, src: u8) -> Instruction {
    Instruction {
        execute: Box::new(move |ctx| {
            let value = ctx.read_reg8(src);
            ctx.write_reg8(dst, value);
        }),
        disassembly: format!("ld {}, {}", reg_to_str(dst), reg_to_str(src)),
    }
}

fn load_reg8_mem(dst: u8) -> Instruction {
    Instruction {
        execute: Box::new(move |ctx| {
            let address = ctx.read_reg16(REG_HL);
            let value = ctx.read_u8(address);
            ctx.write_reg8(dst, value);
        }),
        disassembly: format!("ld {}, (hl)", reg_to_str(dst)),
    }
}

fn load_reg8_const(dst: u8) -> Instruction {
    Instruction {
        execute: Box::new(move |ctx| {
            let value = ctx.load_u8_const();
            ctx.write_reg8(dst, value);
        }),
        disassembly: format!("ld {}, n", reg_to_str(dst)),
    }
}

fn load_reg16_const(dst: u8) -> Instruction {
    Instruction {
        execute: Box::new(move |ctx| {
            let value = ctx.load_u16_const();
            ctx.write_reg16(dst, value);
        }),
        disassembly: format!("ld {}, nn", reg_to_str(dst)),
    }
}

fn illegal_opcode(opcode: u8) -> Instruction {
    Instruction {
        execute: Box::new(move |_| {
            panic!("Invalid opcode {opcode:02x}");
        }),
        disassembly: String::from("illegal"),
    }
}

#[bitmatch]
fn resolve_opcode(opcode: u8) -> Instruction {
    #[bitmatch]
    match opcode {
        "01dd_d110" => load_reg8_mem(d),
        "01dd_dsss" => load_reg8(d, s),
        "00dd_d110" => load_reg8_const(d),
        "00dd_d110" => load_reg16_const(d),
        _ => illegal_opcode(opcode),
    }
}

fn build_instruction_table() -> Box<[Instruction]> {
    (0..u8::MAX)
        .map(resolve_opcode)
        .collect::<Vec<Instruction>>()
        .into_boxed_slice()
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ADDRESS: u16 = 0xDEAD;
    const TEST_VALUE_U8: u8 = 0xBE;
    const TEST_VALUE_U8_2: u8 = 0xEF;

    #[test]
    fn build_instruction_table_test() {
        let table = build_instruction_table();
        assert_eq!(table.len(), 0xff);
    }

    #[test]
    fn load_reg8_mem_test() {
        let mut ctx = Context::default();
        ctx.write_reg16(REG_HL, TEST_ADDRESS);
        ctx.write_u8(TEST_ADDRESS, TEST_VALUE_U8);

        (resolve_opcode(0b0111_1110).execute)(&mut ctx);
        assert_eq!(ctx.read_reg8(REG_A), TEST_VALUE_U8);
    }

    #[test]
    fn load_reg8_test() {
        let mut ctx = Context::default();
        ctx.write_reg8(REG_A, TEST_VALUE_U8);
        ctx.write_reg8(REG_B, TEST_VALUE_U8_2);

        (resolve_opcode(0b0100_0111).execute)(&mut ctx);
        assert_eq!(ctx.read_reg8(REG_B), TEST_VALUE_U8);
    }
}
