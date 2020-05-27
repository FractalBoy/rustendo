#[cfg(test)]
mod tests {
    #[test]
    fn new_mos_6502() {
        assert_eq!(u8::MAX.wrapping_add(1), 0);
    }
}

pub mod mos6502;
