pub const SIGN_BIT: u8 = 7;
pub const ZERO_BIT: u8 = 6;
pub const HALF_CARRY_BIT: u8 = 4;
pub const PARITY_OVERFLOW_BIT: u8 = 2;
pub const SUBTRACT_BIT: u8 = 1;
pub const CARRY_BIT: u8 = 0;

pub fn alu_add(left: u8, right: u8) -> (u8, u8) {
    let (result, carry) = left.overflowing_add(right);

    let mut flags = 0;
    set_bit(&mut flags, SIGN_BIT, left & 0b1000_0000 != 0);
    set_bit(&mut flags, ZERO_BIT, result == 0);
    set_bit(
        &mut flags,
        HALF_CARRY_BIT,
        ((left & 0xf) + (right & 0xf)) > 0xf,
    );
    set_bit(
        &mut flags,
        PARITY_OVERFLOW_BIT,
        (left as i8).overflowing_add(right as i8).1,
    );
    set_bit(&mut flags, SUBTRACT_BIT, false);
    set_bit(&mut flags, CARRY_BIT, carry);

    (result, flags)
}

pub fn set_bit(value: &mut u8, bit: u8, enabled: bool) {
    if enabled {
        *value |= 1 << bit;
    } else {
        *value &= !(1 << bit);
    }
}
