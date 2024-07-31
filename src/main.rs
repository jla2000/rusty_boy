use bitmatch::bitmatch;
use std::{collections::HashMap, sync::LazyLock};

struct Context {
    regs: [u16; 7],
}

fn nop(_: &mut Context) -> TStates {
    (1, 4)
}

type TStates = (u8, u8);
type OpcodeImpl = Box<dyn Fn(&mut Context) -> TStates>;

fn implement_ld(dest: usize, source: usize) -> OpcodeImpl {
    Box::new(move |ctx: &mut Context| {
        ctx.regs[dest] = ctx.regs[source];
        (1, 4)
    })
}

#[bitmatch]
fn implement_opcode(opcode: u8) -> OpcodeImpl {
    #[bitmatch]
    match opcode {
        "01dd_dsss" => implement_ld(d as usize, s as usize),
        _ => Box::new(nop),
    }
}

fn main() {
    let opcode_table = (0..u8::MAX)
        .map(|opcode| (opcode, implement_opcode(opcode)))
        .collect::<HashMap<u8, OpcodeImpl>>();

    opcode_table[&0x00](&mut Context { regs: [0; 7] });
}
