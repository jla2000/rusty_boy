use crate::{Cpu, Reg16};

pub struct Instruction {
    pub execute: Box<dyn Fn(&mut Cpu)>,
    #[allow(clippy::type_complexity)]
    pub disassemble: Box<dyn Fn(&dyn Disassembler) -> String>,
}

impl Instruction {
    #[inline]
    pub fn new<E, D>(execute: E, disassemble: D) -> Self
    where
        E: Fn(&mut Cpu) + 'static,
        D: Fn(&dyn Disassembler) -> String + 'static,
    {
        Self {
            execute: Box::new(execute),
            disassemble: Box::new(disassemble),
        }
    }
}

pub trait Disassembler {
    fn address(&self) -> String;
    fn peek_u8(&self) -> String;
    fn peek_u16(&self) -> String;
}

pub struct StaticDisassembler;
pub struct DynamicDisassembler<'a>(pub &'a mut Cpu);

impl Disassembler for StaticDisassembler {
    fn address(&self) -> String {
        String::from("(HL)")
    }
    fn peek_u8(&self) -> String {
        String::from("N")
    }
    fn peek_u16(&self) -> String {
        String::from("NN")
    }
}

impl Disassembler for DynamicDisassembler<'_> {
    fn address(&self) -> String {
        format!("@{:04x}", self.0.read_reg16(Reg16::HL))
    }
    fn peek_u8(&self) -> String {
        format!(
            "${:02x}",
            self.0.memory[(self.0.program_counter + 1) as usize]
        )
    }
    fn peek_u16(&self) -> String {
        format!(
            "${:02x}{:02x}",
            self.0.memory[(self.0.program_counter + 1) as usize],
            self.0.memory[(self.0.program_counter + 2) as usize]
        )
    }
}
