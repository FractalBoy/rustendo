#[cfg(test)]
mod tests {
    use super::mos6502;

    #[test]
    fn adc_no_carry() {
        let mem = create_memory_from_slice(&[
            0x69, 0x1, // ADC $1
            0x69, 0x1, // ADC $1
            0x8D, 0x8, 0x0, // STA $8
        ]);
        let mut mos6502 = mos6502::Mos6502::new(Some(&mem));
        for _ in 0..3 {
            while mos6502.clock() {}
        }
        assert_eq!(mos6502.read_memory_at_address(0x8), 2, "0x1 + 0x1 = 0x2");
    }

    #[test]
    fn adc_with_carry() {
        let mem = create_memory_from_slice(&[
            0x69, 0xFF, // ADC $FF
            0x69, 0xFF, // ADC $FF
            0x8D, 0x8, 0x0, // STA $8
        ]);
        let mut mos6502 = mos6502::Mos6502::new(Some(&mem));
        for _ in 0..3 {
            while mos6502.clock() {}
        }
        assert_eq!(
            mos6502.read_memory_at_address(0x8),
            0xFE,
            "0xFF + 0xFF = 0xFE"
        );
        assert!(
            mos6502.p.borrow_mut().get_carry(),
            "0xFF + 0xFF sets carry flag"
        );
    }

    #[test]
    fn and_eq_zero() {
        let mem = create_memory_from_slice(&[
            0x69, 0xFF, // ADC $FF
            0x29, 0x00, // AND $00
            0x8D, 0x8, 0x0, // STA $8
        ]);
        let mut mos6502 = mos6502::Mos6502::new(Some(&mem));
        for _ in 0..3 {
            while mos6502.clock() {}
        }
        assert_eq!(
            mos6502.read_memory_at_address(0x8),
            0x00,
            "0xFF & 0xFF = 0x00"
        );
        assert!(mos6502.p.borrow().get_zero(), "zero flag set");
        assert!(!mos6502.p.borrow().get_negative(), "negative flag not set");
    }

    #[test]
    fn and_eq_negative() {
        let mem = create_memory_from_slice(&[
            0x69, 0xFF, // ADC $FF
            0x29, 0x80, // AND $80
            0x8D, 0x8, 0x0, // STA $8
        ]);
        let mut mos6502 = mos6502::Mos6502::new(Some(&mem));
        for _ in 0..3 {
            while mos6502.clock() {}
        }
        assert_eq!(
            mos6502.read_memory_at_address(0x8),
            0x80,
            "0xFF & 0x80 = 0x80"
        );
        assert!(!mos6502.p.borrow().get_zero(), "zero flag not set");
        assert!(mos6502.p.borrow().get_negative(), "negative flag set");
    }

    fn create_memory_from_slice(slice: &[u8]) -> Vec<u8> {
        let mut program = vec![0; 0x800];
        program.splice(0..slice.len(), slice.iter().cloned());
        program
    }
}

pub mod mos6502;
