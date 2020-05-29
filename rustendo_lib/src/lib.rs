#[cfg(test)]
mod tests {
    use super::mos6502;

    #[test]
    fn adc_no_carry() {
        let mut mem: Vec<u8> = vec![0; 0x800];
        mem.splice(
            0..8,
            [
                // ADC $1
                0x69, 0x1,
                // ADC $1
                0x69, 0x1,
                // STA $8
                0x8D, 0x8, 0x0,
                // BRK
                0x0
            ].iter().cloned(),
        );
        let mut mos6502 = mos6502::Mos6502::new(Some(&mem));
        mos6502.run();
        mos6502.address_bus.borrow_mut().write_wide(0x8);
        mos6502.internal_ram.write_address();
        assert_eq!(mos6502.internal_ram.read(), 2, "0x1 + 0x1 = 0x2");
    }

    #[test]
    fn adc_with_carry() {
        let mut mem: Vec<u8> = vec![0; 0x800];
        mem.splice(
            0..8,
            [
                // ADC $FF
                0x69, 0xFF,
                // ADC $FF
                0x69, 0xFF,
                // STA $8
                0x8D, 0x8, 0x0,
                // BRK
                0x0
            ].iter().cloned(),
        );
        let mut mos6502 = mos6502::Mos6502::new(Some(&mem));
        mos6502.run();
        mos6502.address_bus.borrow_mut().write_wide(0x8);
        mos6502.internal_ram.write_address();
        assert_eq!(mos6502.internal_ram.read(), 0xFE, "0xFF + 0xFF = 0xFE");
        assert!(mos6502.p.carry, "0xFF + 0xFF sets carry flag");
    }

    #[test]
    fn and_eq_zero() {
        let mut mem: Vec<u8> = vec![0; 0x800];
        mem.splice(
            0..8,
            [
                // ADC $FF
                0x69, 0xFF,
                // AND $00
                0x29, 0x00,
                // STA $8
                0x8D, 0x8, 0x0,
                // BRK
                0x0
            ].iter().cloned(),
        );
        let mut mos6502 = mos6502::Mos6502::new(Some(&mem));
        mos6502.run();
        mos6502.address_bus.borrow_mut().write_wide(0x8);
        mos6502.internal_ram.write_address();
        assert_eq!(mos6502.internal_ram.read(), 0x00, "0xFF & 0xFF = 0x00");
        assert!(mos6502.p.get_zero(), "zero flag set");
        assert!(!mos6502.p.get_negative(), "negative flag not set");
    }

    #[test]
    fn and_eq_negative() {
        let mut mem: Vec<u8> = vec![0; 0x800];
        mem.splice(
            0..8,
            [
                // ADC $FF
                0x69, 0xFF,
                // AND $80
                0x29, 0x80,
                // STA $8
                0x8D, 0x8, 0x0,
                // BRK
                0x0
            ].iter().cloned(),
        );
        let mut mos6502 = mos6502::Mos6502::new(Some(&mem));
        mos6502.run();
        mos6502.address_bus.borrow_mut().write_wide(0x8);
        mos6502.internal_ram.write_address();
        assert_eq!(mos6502.internal_ram.read(), 0x80, "0xFF & 0x80 = 0x80");
        assert!(!mos6502.p.get_zero(), "zero flag not set");
        assert!(mos6502.p.get_negative(), "negative flag set");
    }

}

pub mod mos6502;
