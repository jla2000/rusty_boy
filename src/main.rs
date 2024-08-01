mod alu;
mod cpu;
mod instruction;
mod translate;

use cpu::*;
use instruction::*;
use translate::*;

fn build_instruction_set() -> Box<[Instruction]> {
    (0..=u8::MAX)
        .map(translate_opcode)
        .collect::<Vec<Instruction>>()
        .into_boxed_slice()
}

fn main() {
    let set = build_instruction_set();
    let mut cpu = Cpu::default();

    loop {
        let opcode = cpu.load_u8_const();
        let instruction = set.get(opcode as usize).unwrap();
        let disassembled_instruction = (instruction.disassemble)(&DynamicDisassembler(&mut cpu));

        println!("-> {disassembled_instruction}");
        (instruction.execute)(&mut cpu);

        if let Some(next_pc) = cpu.program_counter.checked_add(1) {
            cpu.program_counter = next_pc;
        } else {
            println!("End of memory reached. Exiting loop");
            break;
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
    fn build_instruction_set_test() {
        let table = build_instruction_set();
        assert_eq!(table.len(), 256);
    }

    #[test]
    fn load_reg8_mem_test() {
        let mut cpu = Cpu::default();
        cpu.write_reg16(Reg16::HL, TEST_ADDRESS);
        cpu.write_u8(TEST_ADDRESS, TEST_VALUE_U8);

        let opcode = translate_opcode(0b0111_1110);

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

        let opcode = translate_opcode(0b0100_0111);
        assert_eq!((opcode.disassemble)(&StaticDisassembler), "ld B, A");
        assert_eq!(
            (opcode.disassemble)(&DynamicDisassembler(&mut cpu)),
            "ld B, A"
        );

        (opcode.execute)(&mut cpu);
        assert_eq!(cpu.read_reg8(Reg8::B), TEST_VALUE_U8);
    }
}
