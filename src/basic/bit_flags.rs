
#[inline]
pub fn get_bit(number:u8, bit_place:u8) -> bool {
    number & bit_mask(bit_place) != 0
}


#[inline]
#[allow(unused)]
pub fn set_bit(number:&mut u8, value:bool, bit_place:u8) {
    let mask = bit_mask(bit_place);
    match value {
        true => *number |= mask,
        false => *number &= !mask,
    }
}


fn bit_mask(bit_place:u8) -> u8 {
    match bit_place {
        0 => 0b_1000_0000,
        1 => 0b_0100_0000,
        2 => 0b_0010_0000,
        3 => 0b_0001_0000,
        4 => 0b_0000_1000,
        5 => 0b_0000_0100,
        6 => 0b_0000_0010,
        7 => 0b_0000_0001,
        _ => panic!("Invalid bit place")
    }
}