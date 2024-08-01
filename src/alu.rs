pub const SIGN_BIT: u8 = 7;
pub const ZERO_BIT: u8 = 6;
pub const HALF_CARRY_BIT: u8 = 4;
pub const PARITY_OVERFLOW_BIT: u8 = 2;
pub const SUBTRACT_BIT: u8 = 1;
pub const CARRY_BIT: u8 = 0;

pub struct Flags(u8);

impl Flags {
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

pub fn alu_add(left: u8, right: u8) -> (u8, Flags) {
    let (result, carry) = left.overflowing_add(right);

    let mut flags = Flags(0);
    flags.set(SIGN_BIT, left & 0b1000_0000 != 0);
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
