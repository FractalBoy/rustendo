#[cfg(test)]
mod tests {
    use super::mos6502::Mos6502;

    #[test]
    fn adc_no_carry() {
        let mut mos6502 = run_program(&[
            vec![0x69, 0x1],      // ADC $1
            vec![0x69, 0x1],      // ADC $1
            vec![0x85, 0x8, 0x0], // STA $8
        ]);
        assert_eq!(mos6502.read_memory_at_address(0x8), 2, "0x1 + 0x1 = 0x2");
    }

    #[test]
    fn adc_with_carry() {
        let mut mos6502 = run_program(&[
            vec![0x69, 0xFF],     // ADC $FF
            vec![0x69, 0xFF],     // ADC $FF
            vec![0x85, 0x8, 0x0], // STA $8
        ]);
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
    fn adc_bcd() {
        let mut mos6502 = run_program(&[
            vec![0xF8],       // SED
            vec![0x69, 0x10], // ADC 10
            vec![0x69, 0x10], // ADC 10
            vec![0x85, 0x8],  // STA $8
        ]);
        assert_eq!(
            mos6502.read_memory_at_address(0x8),
            0x20,
            "0x10 + 0x10 = 0x20 in BCD"
        );
        let mut mos6502 = run_program(&[
            vec![0xF8],       // SED
            vec![0x69, 0x81], // ADC 81
            vec![0x69, 0x92], // ADC 92
            vec![0x85, 0x8],  // STA $8
        ]);
        assert_eq!(
            mos6502.read_memory_at_address(0x8),
            0x73,
            "0x81 + 0x92 = 0x73 in BCD"
        );
        assert!(
            mos6502.p.borrow().get_carry(),
            "0x81 + 0x92 sets carry flag"
        );
    }

    #[test]
    fn and_eq_zero() {
        let mut mos6502 = run_program(&[
            vec![0x69, 0xFF], // ADC $FF
            vec![0x29, 0x00], // AND $00
            vec![0x85, 0x8],  // STA $8
        ]);
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
        let mut mos6502 = run_program(&[
            vec![0x69, 0xFF], // ADC $FF
            vec![0x29, 0x80], // AND $80
            vec![0x85, 0x8],  // STA $8
        ]);
        assert_eq!(
            mos6502.read_memory_at_address(0x8),
            0x80,
            "0xFF & 0x80 = 0x80"
        );
        assert!(!mos6502.p.borrow().get_zero(), "zero flag not set");
        assert!(mos6502.p.borrow().get_negative(), "negative flag set");
    }

    #[test]
    fn sbc_without_borrow() {
        let mut mos6502 = run_program(&[
            vec![0x69, 0xF6], // ADC $F6
            vec![0x38],       // SEC (disable borrow)
            vec![0xE9, 0x05], // SBC $5
            vec![0x85, 0x8],  // STA $8
        ]);
        assert_eq!(
            mos6502.read_memory_at_address(0x08),
            0xF1,
            "0xF6 - 0x05 = 0x0F"
        );
        assert!(mos6502.p.borrow().get_carry(), "no borrow");
        assert!(mos6502.p.borrow().get_negative(), "answer is negative");
    }

    #[test]
    fn sbc_with_borrow() {
        let mut mos6502 = run_program(&[
            vec![0x69, 0x5], // ADC $5
            vec![0x38],      // SEC (disable borrow)
            vec![0xE9, 0xA], // SBC $A
            vec![0x85, 0x8], // STA $8
        ]);
        assert_eq!(
            mos6502.read_memory_at_address(0x08),
            0xFB,
            "0x5 - 0xA = -0x5 (0xFB)"
        );
        assert!(!mos6502.p.borrow().get_carry(), "borrow");
        assert!(mos6502.p.borrow().get_negative(), "answer is positive");
    }

    #[test]
    fn sbc_bcd() {
        let mut mos6502 = run_program(&[
            vec![0xF8],       // SED
            vec![0x69, 0x92], // ADC 92
            vec![0x38],       // SEC (disable borrow)
            vec![0xE9, 0x25], // SBC 25
            vec![0x85, 0x8],  // STA $9
        ]);

        assert_eq!(mos6502.read_memory_at_address(0x8), 0x67);
        assert!(!mos6502.p.borrow().get_carry(), "no borrow set");
        assert!(!mos6502.p.borrow().get_negative(), "not negative");

        let mut mos6502 = run_program(&[
            vec![0xF8],       // SED
            vec![0x69, 0x25], // ADC 25 
            vec![0x38],       // SEC (disable borrow)
            vec![0xE9, 0x92], // SBC 92
            vec![0x85, 0x8],  // STA $9
        ]);

        assert_eq!(mos6502.read_memory_at_address(0x8), 0x33);
        assert!(!mos6502.p.borrow().get_carry(), "borrow set");
        assert!(!mos6502.p.borrow().get_negative(), "not negative");
    }

    fn run_program(program: &[Vec<u8>]) -> Mos6502 {
        let mut mem: Vec<u8> = Vec::new();
        for instruction in program.iter().cloned() {
            mem.extend_from_slice(&instruction);
        }
        let mut mos6502 = Mos6502::new(Some(&mem));
        for _ in 0..program.len() {
            while mos6502.clock() {}
        }
        mos6502
    }
}

pub mod mos6502;
