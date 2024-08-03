mod alu;
mod cpu;
mod decode;
mod instruction;

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
        let opcode = cpu.read_mem8(cpu.program_counter);
        let instruction = &instruction_table[opcode as usize];
        let disassembled_instruction = instruction.disassemble();

        println!("State: {}", &cpu);
        println!("{disassembled_instruction}",);
        instruction.execute(&mut cpu);

        if let Some(address) = cpu.jump_address.take() {
            cpu.program_counter = address;
        } else {
            match cpu.program_counter.checked_add(1) {
                Some(pc) => cpu.program_counter = pc,
                None => break,
            }
        }
    }

    println!("Program end reached");
}

#[cfg(test)]
mod tests {
    use alu::{
        Flags, CARRY_BIT, HALF_CARRY_BIT, PARITY_OVERFLOW_BIT, SIGN_BIT, SUBTRACT_BIT, ZERO_BIT,
    };

    use super::*;

    const TEST_ADDRESS: u16 = 0xdead;
    const TEST_VALUE_U8: u8 = 0xbe;
    const TEST_VALUE_U8_2: u8 = 0xef;

    #[test]
    fn build_instruction_table_test() {
        let table = build_instruction_table();
        assert_eq!(table.len(), 256);
    }

    #[test]
    fn load_reg8_mem_test() {
        let mut cpu = Cpu::default();
        cpu.write_reg16(Reg16::HL, TEST_ADDRESS);
        cpu.write_mem8(TEST_ADDRESS, TEST_VALUE_U8);

        let opcode = decode_opcode(0b0111_1110);
        assert_eq!(opcode.disassemble(), "ld a, (hl)");
        opcode.execute(&mut cpu);
        assert_eq!(cpu.read_reg8(Reg8::A), TEST_VALUE_U8);
    }

    #[test]
    fn load_reg8_test() {
        let mut cpu = Cpu::default();
        cpu.write_reg8(Reg8::A, TEST_VALUE_U8);
        cpu.write_reg8(Reg8::B, TEST_VALUE_U8_2);

        let opcode = decode_opcode(0b0100_0111);
        assert_eq!(opcode.disassemble(), "ld b, a");

        opcode.execute(&mut cpu);
        assert_eq!(cpu.read_reg8(Reg8::B), TEST_VALUE_U8);
    }

    #[test]
    fn load_reg8_const_test() {
        let mut cpu = Cpu::default();
        cpu.write_mem8(0x0000, TEST_VALUE_U8);

        let opcode = decode_opcode(0b0000_0110);
        assert_eq!(opcode.disassemble(), "ld b, n");

        opcode.execute(&mut cpu);
        assert_eq!(cpu.read_reg8(Reg8::B), TEST_VALUE_U8);
    }

    #[test]
    fn add_test() {
        let mut cpu = Cpu::default();
        cpu.write_reg8(Reg8::A, 0xf0);
        cpu.write_reg8(Reg8::B, 0x0f);

        let opcode = decode_opcode(0b1000_0000);
        assert_eq!(opcode.disassemble(), "add a, b");

        opcode.execute(&mut cpu);
        assert_eq!(cpu.read_reg8(Reg8::A), 0xff);

        let flags = Flags::from(cpu.read_reg8(Reg8::F));
        assert_eq!(flags.get(SIGN_BIT), true);
        assert_eq!(flags.get(ZERO_BIT), false);
        assert_eq!(flags.get(HALF_CARRY_BIT), false);
        assert_eq!(flags.get(PARITY_OVERFLOW_BIT), false);
        assert_eq!(flags.get(SUBTRACT_BIT), false);
        assert_eq!(flags.get(CARRY_BIT), false);
    }

    #[test]
    fn add_overflow_test() {
        let mut cpu = Cpu::default();
        cpu.write_reg8(Reg8::A, 0xff);
        cpu.write_reg8(Reg8::B, 0x01);

        let opcode = decode_opcode(0b1000_0000);
        assert_eq!(opcode.disassemble(), "add a, b");

        opcode.execute(&mut cpu);
        assert_eq!(cpu.read_reg8(Reg8::A), 0x00);

        let flags = Flags::from(cpu.read_reg8(Reg8::F));
        assert_eq!(flags.get(SIGN_BIT), false);
        assert_eq!(flags.get(ZERO_BIT), true);
        assert_eq!(flags.get(HALF_CARRY_BIT), true);
        assert_eq!(flags.get(PARITY_OVERFLOW_BIT), false);
        assert_eq!(flags.get(SUBTRACT_BIT), false);
        assert_eq!(flags.get(CARRY_BIT), true);
    }

    #[test]
    fn jump_test() {
        let mut cpu = Cpu::default();
        cpu.write_mem16(0x0000, TEST_ADDRESS);

        let opcode = decode_opcode(0b1100_0011);
        assert_eq!(opcode.disassemble(), "jp nn");

        assert_eq!(cpu.jump_address, None);
        opcode.execute(&mut cpu);
        assert_eq!(cpu.jump_address, Some(TEST_ADDRESS));
    }

    #[test]
    fn jump_indirect_test() {
        let mut cpu = Cpu::default();
        cpu.write_reg16(Reg16::HL, TEST_ADDRESS);

        let opcode = decode_opcode(0b1100_1011);
        assert_eq!(opcode.disassemble(), "jp (hl)");

        assert_eq!(cpu.jump_address, None);
        opcode.execute(&mut cpu);
        assert_eq!(cpu.jump_address, Some(TEST_ADDRESS));
    }
}
