use bitmatch::bitmatch;
use std::{collections::HashMap, sync::LazyLock};

struct Context {
    regs: [u8; 16],
}

impl Context {
    fn read_reg8(&self, reg: u8) -> u8 {
        self.regs[reg as usize]
    }

    fn write_reg8(&mut self, reg: u8, value: u8) {
        self.regs[reg as usize] = value;
    }
}

fn nop(_: &mut Context) -> TStates {
    (1, 4)
}

type TStates = (u8, u8);
type OpcodeImpl = Box<dyn Fn(&mut Context) -> TStates>;

fn ld8(dst: u8, src: u8) -> OpcodeImpl {
    Box::new(move |ctx| {
        let val = ctx.read_reg8(src);
        ctx.write_reg8(dst, val);
        (1, 4)
    })
}

fn ld16(dst: u16, src: u16) -> OpcodeImpl {
    Box::new(move |ctx| {
        let val = ctx.read_reg8(src);
        ctx.write_reg8(dst, val);
        (1, 4)
    })
}

#[bitmatch]
fn implement_opcode(opcode: u8) -> OpcodeImpl {
    #[bitmatch]
    match opcode {
        "01dd_dsss" => ld8(d.into(), s.into()),
        "00dd_d110" => ld_const(d.into()),
        _ => Box::new(nop),
    }
}

fn main() {
    let opcode_table = (0..u8::MAX)
        .map(|opcode| (opcode, implement_opcode(opcode)))
        .collect::<HashMap<u8, OpcodeImpl>>();

    opcode_table[&0x00](&mut Context { regs: [0; 16] });
}
