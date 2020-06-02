#[cfg(test)]
mod tests {
    use super::rp2a03::Rp2a03;

    #[test]
    fn adc_no_carry() {
        let mut rp2a03 = run_program(&[
            vec![0xA9, 0x1],  // LDA #$1
            vec![0x69, 0x1],  // ADC #$1
            vec![0x85, 0xD],  // STA $D
            vec![0xA9, 0x00], // LDA #$00
            vec![0x69, 0x0],  // ADC #$0
            vec![0x85, 0xE],  // STA $E
        ]);
        assert_eq!(rp2a03.read_memory_at_address(0xD), 2, "0x1 + 0x1 = 0x2");
        assert_eq!(rp2a03.read_memory_at_address(0xE), 0, "carry bit cleared");
    }

    #[test]
    fn adc_with_carry() {
        let mut rp2a03 = run_program(&[
            vec![0x69, 0xFF], // ADC $FF
            vec![0x69, 0xFF], // ADC $FF
            vec![0x85, 0xD],  // STA $D
            vec![0x29, 0x00], // AND $00 (clear accumulator)
            vec![0x69, 0x0],  // ADC $0
            vec![0x85, 0xE],  // STA $E
        ]);
        assert_eq!(
            rp2a03.read_memory_at_address(0xD),
            0xFE,
            "0xFF + 0xFF = 0xFE"
        );
        assert_eq!(
            rp2a03.read_memory_at_address(0xE),
            0x1,
            "0xFF + 0xFF sets carry flag"
        );
    }

    #[test]
    fn adc_bcd() {
        let mut rp2a03 = run_program(&[
            vec![0xF8],       // SED
            vec![0xA9, 0x10], // LDA #$10
            vec![0x69, 0x10], // ADC #$10
            vec![0x85, 0xD],  // STA $D
            vec![0xA9, 0x00], // LDA #$0
            vec![0x69, 0x0],  // ADC #$0
            vec![0x85, 0xE],  // STA $E
        ]);
        assert_eq!(
            rp2a03.read_memory_at_address(0xD),
            0x20,
            "0x10 + 0x10 = 0x20 in BCD"
        );
        assert_eq!(rp2a03.read_memory_at_address(0xE), 0x0, "carry bit cleared");
        let mut rp2a03 = run_program(&[
            vec![0xF8],       // SED
            vec![0xA9, 0x81], // LDA #$81
            vec![0x69, 0x92], // ADC #$92
            vec![0x85, 0xD],  // STA $D
            vec![0xA9, 0x0],  // LDA #$0
            vec![0x69, 0x0],  // ADC #$0
            vec![0x85, 0xE],  // STA $E
        ]);
        assert_eq!(
            rp2a03.read_memory_at_address(0xD),
            0x73,
            "0x81 + 0x92 = 0x73 in BCD"
        );
        assert_eq!(
            rp2a03.read_memory_at_address(0xE),
            0x1,
            "0x81 + 0x92 sets carry flag"
        );
    }

    #[test]
    fn and_eq_zero() {
        let mut rp2a03 = run_program(&[
            vec![0x69, 0xFF], // ADC #$FF
            vec![0x29, 0x00], // AND #$00
            vec![0x30, 0x2],  // BMI $2
            vec![0xF0, 0x2],  // BEQ $2
            vec![0x69, 0x2],  // ADC #$2 (should never happen)
            vec![0x69, 0x1],  // ADC #$1 (should branch here from BEQ)
            vec![0x85, 0xD],  // STA $D
        ]);
        assert_eq!(
            rp2a03.read_memory_at_address(0xD),
            0x1,
            "((0xFF & 0xFF) + 0x01)= 0x01"
        );
    }

    #[test]
    fn and_eq_negative() {
        let mut rp2a03 = run_program(&[
            vec![0x69, 0xFF], // ADC $FF
            vec![0x29, 0x80], // AND $80
            vec![0xF0, 0x2],  // BEQ $2
            vec![0x30, 0x2],  // BMI $2
            vec![0x69, 0x2],  // ADC $2 (should never happen)
            vec![0x69, 0x1],  // ADC $1 (should branch here from BMI)
            vec![0x85, 0xD],  // STA $D
        ]);
        assert_eq!(
            rp2a03.read_memory_at_address(0xD),
            0x81,
            "(0xFF & 0x80) + 0x01 = 0x81"
        );
    }

    #[test]
    fn sbc_without_borrow() {
        let mut rp2a03 = run_program(&[
            vec![0x69, 0x76], // ADC $76
            vec![0x38],       // SEC (disable borrow)
            vec![0xE9, 0x05], // SBC $5
            vec![0x30, 0x8],  // BMI $8 (should not be taken)
            vec![0x85, 0xF],  // STA $F
            vec![0x29, 0x00], // AND $00 (clear accumulator)
            vec![0x69, 0x0],  // ADC $0
            vec![0x85, 0x10], // STA $10
        ]);
        assert_eq!(
            rp2a03.read_memory_at_address(0xF),
            0x71,
            "0x76 - 0x05 = 0x71, BMI branch not taken"
        );
        assert_eq!(
            rp2a03.read_memory_at_address(0x10),
            0x1,
            "no borrow (carry set)"
        );
    }

    #[test]
    fn sbc_with_borrow() {
        let mut rp2a03 = run_program(&[
            vec![0x69, 0x5],  // ADC $5
            vec![0x38],       // SEC (disable borrow)
            vec![0xE9, 0xA],  // SBC $A
            vec![0x10, 0x8],  // BPL $9 (should not be taken)
            vec![0x85, 0xF],  // STA $F
            vec![0x29, 0x00], // AND $00 (clear accumulator)
            vec![0x69, 0x0],  // ADC $0
            vec![0x85, 0x10], // STA $10
        ]);
        assert_eq!(
            rp2a03.read_memory_at_address(0xF),
            0xFB,
            "0x5 - 0xA = -0x5 (0xFB), BPL branch not taken"
        );
        assert_eq!(
            rp2a03.read_memory_at_address(0x10),
            0x0,
            "borrow (carry not set)"
        );
    }

    #[test]
    fn sbc_bcd() {
        let mut rp2a03 = run_program(&[
            vec![0xF8],       // SED
            vec![0x69, 0x92], // ADC 92
            vec![0x38],       // SEC (disable borrow)
            vec![0xE9, 0x25], // SBC 25
            vec![0x30, 0x8],  // BMI $4 (should not be taken)
            vec![0x85, 0x10], // STA $10
            vec![0x29, 0x00], // AND $00 (clear accumulator)
            vec![0x69, 0x0],  // ADC $0
            vec![0x85, 0x11], // STA $11
        ]);

        assert_eq!(rp2a03.read_memory_at_address(0x10), 0x67);
        assert_eq!(rp2a03.read_memory_at_address(0x11), 0x0);

        //let mut rp2a03 = run_program(&[
        //    vec![0xF8],       // SED
        //    vec![0x69, 0x25], // ADC 25
        //    vec![0x38],       // SEC (disable borrow)
        //    vec![0xE9, 0x92], // SBC 92
        //    vec![0x85, 0x8],  // STA $9
        //]);

        //assert_eq!(rp2a03.read_memory_at_address(0x8), 0x33);
        //assert!(!rp2a03.p.borrow().get_carry(), "borrow set");
        //assert!(!rp2a03.p.borrow().get_negative(), "not negative");
    }

    fn run_program(program: &[Vec<u8>]) -> Rp2a03 {
        let mut mem: Vec<u8> = Vec::new();
        for instruction in program.iter().cloned() {
            mem.extend_from_slice(&instruction);
        }
        let mut rp2a03 = Rp2a03::new(Some(&mem));
        for _ in 0..program.len() + 1 {
            while rp2a03.clock() {}
        }
        rp2a03
    }
}

mod bus;
mod mos6502;
mod ram;
pub mod rp2a03;
