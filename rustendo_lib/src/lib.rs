#[cfg(test)]
mod tests {
    use super::mos6502;

    #[test]
    fn adc() {
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
        mos6502.address_bus.write_wide(0x8);
        mos6502.internal_ram.write_address(&mos6502.address_bus);
        assert_eq!(mos6502.internal_ram.read(), 2);
    }
}

pub mod mos6502;
