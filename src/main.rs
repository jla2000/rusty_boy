mod alu;
mod cpu;
mod decode;
mod instruction;

use std::io::{stdout, Write};

use cpu::*;
use decode::*;
use instruction::*;

fn build_instruction_table() -> Box<[Instruction]> {
    (0..=u8::MAX)
        .map(decode_opcode)
        .collect::<Vec<Instruction>>()
        .into_boxed_slice()
}

fn main() {
    let instruction_table = build_instruction_table();
    let mut cpu = Cpu::default();

    loop {
        let opcode = cpu.read_u8(cpu.program_counter);
        let instruction = &instruction_table[opcode as usize];
        let disassembled_instruction = instruction.disassemble();

        println!("-> {disassembled_instruction}");
        instruction.execute(&mut cpu);

        if let Some(address) = cpu.jump_address.take() {
            cpu.program_counter = address;
        } else {
            match cpu.program_counter.overflowing_add(1) {
                (new_pc, false) => cpu.program_counter = new_pc,
                (_, true) => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ADDRESS: u16 = 0xDEAD;
    const TEST_VALUE_U8: u8 = 0xBE;
    const TEST_VALUE_U8_2: u8 = 0xEF;

    #[test]
    fn build_instruction_table_test() {
        let table = build_instruction_table();
        assert_eq!(table.len(), 256);
    }

    #[test]
    fn load_reg8_mem_test() {
        let mut cpu = Cpu::default();
        cpu.write_reg16(Reg16::HL, TEST_ADDRESS);
        cpu.write_u8(TEST_ADDRESS, TEST_VALUE_U8);

        let opcode = decode_opcode(0b0111_1110);

        assert_eq!((opcode.disassemble)(&StaticDisassembler), "ld A, (HL)");
        assert_eq!(
            (opcode.disassemble)(&DynamicDisassembler(&mut cpu)),
            "ld A, @dead"
        );

        (opcode.execute)(&mut cpu);
        assert_eq!(cpu.read_reg8(Reg8::A), TEST_VALUE_U8);
    }

    #[test]
    fn load_reg8_test() {
        let mut cpu = Cpu::default();
        cpu.write_reg8(Reg8::A, TEST_VALUE_U8);
        cpu.write_reg8(Reg8::B, TEST_VALUE_U8_2);

        let opcode = decode_opcode(0b0100_0111);
        assert_eq!((opcode.disassemble)(&StaticDisassembler), "ld B, A");
        assert_eq!(
            (opcode.disassemble)(&DynamicDisassembler(&mut cpu)),
            "ld B, A"
        );

        (opcode.execute)(&mut cpu);
        assert_eq!(cpu.read_reg8(Reg8::B), TEST_VALUE_U8);
    }
}
