use core::fmt;
use std::fmt::Write;

pub const SIGN_BIT: u8 = 7;
pub const ZERO_BIT: u8 = 6;
pub const HALF_CARRY_BIT: u8 = 4;
pub const PARITY_OVERFLOW_BIT: u8 = 2;
pub const SUBTRACT_BIT: u8 = 1;
pub const CARRY_BIT: u8 = 0;

pub struct Flags(u8);

impl Flags {
    pub fn from(value: u8) -> Self {
        Self(value)
    }

    pub fn set(&mut self, flag_bit: u8, enabled: bool) {
        if enabled {
            self.0 |= 1 << flag_bit;
        } else {
            self.0 &= !(1 << flag_bit);
        }
    }
    pub fn get(&self, flag_bit: u8) -> bool {
        self.0 & (1 << flag_bit) != 0
    }
}

impl From<Flags> for u8 {
    fn from(val: Flags) -> Self {
        val.0
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.get(SIGN_BIT) as u8;
        let z = self.get(ZERO_BIT) as u8;
        let h = self.get(HALF_CARRY_BIT) as u8;
        let pv = self.get(PARITY_OVERFLOW_BIT) as u8;
        let n = self.get(SUBTRACT_BIT) as u8;
        let c = self.get(CARRY_BIT) as u8;
        f.write_fmt(format_args!("|S:{s} Z:{z} H:{h} PV: {pv} N:{n} C:{c}|"))
    }
}

pub fn alu_add(left: u8, right: u8) -> (u8, Flags) {
    let (result, carry) = left.overflowing_add(right);

    let mut flags = Flags(0);
    flags.set(SIGN_BIT, result & 0b1000_0000 != 0);
    flags.set(ZERO_BIT, result == 0);
    flags.set(HALF_CARRY_BIT, ((left & 0xf) + (right & 0xf)) > 0xf);
    flags.set(
        PARITY_OVERFLOW_BIT,
        (left as i8).overflowing_add(right as i8).1,
    );
    flags.set(SUBTRACT_BIT, false);
    flags.set(CARRY_BIT, carry);

    (result, flags)
}
