use bitmatch::bitmatch;
use std::{collections::HashMap, sync::LazyLock};

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

fn nop(_: &mut Context) -> TStates {
    (1, 4)
}

type TStates = (u8, u8);
type OpcodeImpl = Box<dyn Fn(&mut Context) -> TStates>;

fn load_reg8(dst: u8, src: u8) -> OpcodeImpl {
    Box::new(move |ctx| {
        let val = ctx.read_reg8(src);
        ctx.write_reg8(dst, val);
        (1, 4)
    })
}

fn load_reg16_from_const(dst: u8) -> OpcodeImpl {
    Box::new(move |ctx| {
        let val = ctx.load_u16_const();
        ctx.write_reg16(dst, val);
        (1, 4)
    })
}

#[bitmatch]
fn resolve_opcode(opcode: u8) -> OpcodeImpl {
    #[bitmatch]
    match opcode {
        "01dd_dsss" => load_reg8(d.into(), s.into()),
        "00dd_d110" => load_reg16_from_const(d.into()),
        _ => Box::new(nop),
    }
}

fn main() {
    let opcode_table = (0..u8::MAX)
        .map(|opcode| (opcode, resolve_opcode(opcode)))
        .collect::<HashMap<u8, OpcodeImpl>>();

    opcode_table[&0x00](&mut Context {
        general_purpose_regs: [0; 16],
        program_counter: 0,
        stack_pointer: 0,
        index_x: 0,
        index_y: 0,
        memory_refresh: 0,
        memory: [0; 0xffff],
    });
}
