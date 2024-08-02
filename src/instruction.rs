use crate::Cpu;

pub struct Instruction {
    execute: Box<dyn Fn(&mut Cpu)>,
    disassembly: Box<str>,
}

impl Instruction {
    #[inline]
    pub fn new<E>(execute: E, disassembly: impl Into<Box<str>>) -> Self
    where
        E: Fn(&mut Cpu) + 'static,
    {
        Self {
            execute: Box::new(execute),
            disassembly: disassembly.into(),
        }
    }

    pub fn execute(&self, cpu: &mut Cpu) {
        (self.execute)(cpu)
    }

    pub fn disassemble(&self) -> &str {
        self.disassembly.as_ref()
    }
}
