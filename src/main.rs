use bitmatch::bitmatch;

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

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
enum Reg8 {
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
enum Reg16 {
    BC = 0b00,
    DE = 0b01,
    HL = 0b10,
    SP = 0b11,
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

impl Context {
    fn read_reg8(&self, reg: Reg8) -> u8 {
        self.general_purpose_regs[reg as usize]
    }

    fn read_reg16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::BC => u16::from_be_bytes([self.read_reg8(Reg8::B), self.read_reg8(Reg8::C)]),
            Reg16::DE => u16::from_be_bytes([self.read_reg8(Reg8::D), self.read_reg8(Reg8::E)]),
            Reg16::HL => self.read_u16(u16::from_be_bytes([
                self.read_reg8(Reg8::H),
                self.read_reg8(Reg8::L),
            ])),
            Reg16::SP => self.stack_pointer,
        }
    }

    fn write_reg16(&mut self, reg: Reg16, value: u16) {
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
                self.write_u16(address, value);
            }
            Reg16::SP => {
                self.stack_pointer = value;
            }
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

    fn write_reg8(&mut self, reg: Reg8, value: u8) {
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
    #[allow(clippy::type_complexity)]
    pub disassemble: Box<dyn Fn(&dyn Disassembler) -> String>,
}

impl Instruction {
    #[inline]
    fn new<E, D>(execute: E, disassemble: D) -> Self
    where
        E: Fn(&mut Context) + 'static,
        D: Fn(&dyn Disassembler) -> String + 'static,
    {
        Self {
            execute: Box::new(execute),
            disassemble: Box::new(disassemble),
        }
    }
}

trait Disassembler {
    fn address(&self) -> String;
    fn peek_u8(&self) -> String;
    fn peek_u16(&self) -> String;
}

struct StaticDisassembler;
struct DynamicDisassembler<'a>(&'a mut Context);

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

fn alu_add(left: u8, right: u8) -> (u8, u8) {
    let result = left as u16 + right as u16;

    let mut flags = 0;
    // Sign
    flags |= (result & 0b1000_0000) as u8;
    // Zero
    flags |= (1 << 6) * (result == 0) as u8;
    // Carry
    flags |= (result & 0b1_0000_0000) as u8;

    (result as u8, flags)
}

fn load_reg8(dst: Reg8, src: Reg8) -> Instruction {
    Instruction::new(
        move |ctx| {
            let value = ctx.read_reg8(src);
            ctx.write_reg8(dst, value);
        },
        move |_| format!("ld {dst:?}, {src:?}"),
    )
}

fn load_reg8_indirect(dst: Reg8) -> Instruction {
    Instruction::new(
        move |ctx| {
            let address = ctx.read_reg16(Reg16::HL);
            let value = ctx.read_u8(address);
            ctx.write_reg8(dst, value);
        },
        move |disassembler| format!("ld {dst:?}, {}", disassembler.address()),
    )
}

fn load_reg8_const(dst: Reg8) -> Instruction {
    Instruction::new(
        move |ctx| {
            let value = ctx.load_u8_const();
            ctx.write_reg8(dst, value);
        },
        move |disassembler| format!("ld {dst:?}, {}", disassembler.peek_u8()),
    )
}

fn load_reg16_const(dst: Reg16) -> Instruction {
    Instruction::new(
        move |ctx| {
            let value = ctx.load_u16_const();
            ctx.write_reg16(dst, value);
        },
        move |disassembler| format!("ld {dst:?}, {}", disassembler.peek_u16()),
    )
}

fn add_a(src: Reg8) -> Instruction {
    Instruction::new(
        move |ctx| {
            let (value, flags) = alu_add(ctx.read_reg8(Reg8::A), ctx.read_reg8(src));
            // TODO: implement proper flags
            ctx.write_reg8(Reg8::F, flags);
            ctx.write_reg8(Reg8::A, value);
        },
        move |_| format!("add A, {src:?}"),
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

#[bitmatch]
fn resolve_opcode(opcode: u8) -> Instruction {
    #[bitmatch]
    match opcode {
        "01dd_d110" => load_reg8_indirect(d.into()),
        "01dd_dsss" => load_reg8(d.into(), s.into()),
        "00dd_d110" => load_reg8_const(d.into()),
        "00dd_0001" => load_reg16_const(d.into()),
        "1000_0ddd" => add_a(d.into()),
        _ => illegal_opcode(opcode),
    }
}

fn build_instruction_table() -> Box<[Instruction]> {
    (0..u8::MAX)
        .map(resolve_opcode)
        .collect::<Vec<Instruction>>()
        .into_boxed_slice()
}

fn main() {
    let table = build_instruction_table();
    for opcode in 0..u8::MAX {
        println!(
            "0x{opcode:02x}: {}",
            (table[opcode as usize].disassemble)(&StaticDisassembler)
        );
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
        assert_eq!(table.len(), 0xff);
    }

    #[test]
    fn load_reg8_mem_test() {
        let mut ctx = Context::default();
        ctx.write_reg16(Reg16::HL, TEST_ADDRESS);
        ctx.write_u8(TEST_ADDRESS, TEST_VALUE_U8);

        let opcode = resolve_opcode(0b0111_1110);

        assert_eq!((opcode.disassemble)(&StaticDisassembler), "ld A, (HL)");
        assert_eq!(
            (opcode.disassemble)(&DynamicDisassembler(&mut ctx)),
            "ld A, @dead"
        );

        (opcode.execute)(&mut ctx);
        assert_eq!(ctx.read_reg8(Reg8::A), TEST_VALUE_U8);
    }

    #[test]
    fn load_reg8_test() {
        let mut ctx = Context::default();
        ctx.write_reg8(Reg8::A, TEST_VALUE_U8);
        ctx.write_reg8(Reg8::B, TEST_VALUE_U8_2);

        let opcode = resolve_opcode(0b0100_0111);
        assert_eq!((opcode.disassemble)(&StaticDisassembler), "ld B, A");
        assert_eq!(
            (opcode.disassemble)(&DynamicDisassembler(&mut ctx)),
            "ld B, A"
        );

        (opcode.execute)(&mut ctx);
        assert_eq!(ctx.read_reg8(Reg8::B), TEST_VALUE_U8);
    }
}
