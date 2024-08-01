use bitmatch::bitmatch;

use crate::alu::*;
use crate::{Instruction, Reg16, Reg8};

#[bitmatch]
pub fn translate_opcode(opcode: u8) -> Instruction {
    #[bitmatch]
    match opcode {
        "0000_0000" => nop(),
        "01dd_d110" => load_reg8_indirect(d.into()),
        "01dd_dsss" => load_reg8(d.into(), s.into()),
        "00dd_d110" => load_reg8_const(d.into()),
        "00dd_0001" => load_reg16_const(d.into()),
        "1000_0ddd" => add_a(d.into()),
        "1100_0011" => jump(),
        _ => illegal_opcode(opcode),
    }
}

fn nop() -> Instruction {
    Instruction::new(move |_| {}, move |_| String::from("nop"))
}

fn load_reg8(dst: Reg8, src: Reg8) -> Instruction {
    Instruction::new(
        move |cpu| {
            let value = cpu.read_reg8(src);
            cpu.write_reg8(dst, value);
        },
        move |_| format!("ld {dst:?}, {src:?}"),
    )
}

fn load_reg8_indirect(dst: Reg8) -> Instruction {
    Instruction::new(
        move |cpu| {
            let address = cpu.read_reg16(Reg16::HL);
            let value = cpu.read_u8(address);
            cpu.write_reg8(dst, value);
        },
        move |disassembler| format!("ld {dst:?}, {}", disassembler.address()),
    )
}

fn load_reg8_const(dst: Reg8) -> Instruction {
    Instruction::new(
        move |cpu| {
            let value = cpu.load_u8_const();
            cpu.write_reg8(dst, value);
        },
        move |disassembler| format!("ld {dst:?}, {}", disassembler.peek_u8()),
    )
}

fn load_reg16_const(dst: Reg16) -> Instruction {
    Instruction::new(
        move |cpu| {
            let value = cpu.load_u16_const();
            cpu.write_reg16(dst, value);
        },
        move |disassembler| format!("ld {dst:?}, {}", disassembler.peek_u16()),
    )
}

fn add_a(src: Reg8) -> Instruction {
    Instruction::new(
        move |cpu| {
            let (result, flags) = alu_add(cpu.read_reg8(Reg8::A), cpu.read_reg8(src));
            cpu.write_reg8(Reg8::A, result);
            cpu.write_reg8(Reg8::F, flags.into());
        },
        move |_| format!("add A, {src:?}"),
    )
}

fn jump() -> Instruction {
    Instruction::new(
        move |cpu| {
            let address = cpu.load_u16_const();
            cpu.program_counter = address;
        },
        move |disassembler| format!("jump {}", disassembler.peek_u16()),
    )
}

fn illegal_opcode(opcode: u8) -> Instruction {
    Instruction::new(
        move |_| {
            panic!("Invalid opcode {opcode:02x}");
        },
        move |_| String::from("illegal"),
    )
}
