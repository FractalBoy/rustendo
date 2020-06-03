#[cfg(test)]
mod tests {
    use super::assembler::run_program;

    #[test]
    fn adc() {
        let mut cpu = run_program(
            "
        LDA #$01
        ADC #$01
        STA $FF
        PHP
        ",
        );
        assert_eq!(cpu.read_memory_at_address(0xFF), 2, "0x1 + 0x1 = 0x2");
        assert_eq!(
            cpu.read_memory_at_address(0x01FD) & 0x01,
            0x00,
            "carry bit cleared"
        );

        let mut cpu = run_program(
            "
            LDA #$FF
            ADC #$FF
            STA $FF
            PHP
        ",
        );
        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFE, "0xFF + 0xFF = 0xFE");
        assert_eq!(
            cpu.read_memory_at_address(0x01FD) & 0x01,
            0x01,
            "carry bit set"
        );
    }

    #[test]
    fn adc_bcd() {
        let mut cpu = run_program(
            "
            SED
            LDA #$10
            ADC #$10
            STA $FF
            PHP
        ",
        );
        assert_eq!(
            cpu.read_memory_at_address(0xFF),
            0x20,
            "0x10 + 0x10 = 0x20 in BCD"
        );
        assert_eq!(
            cpu.read_memory_at_address(0x01FD) & 0x01,
            0x00,
            "carry bit cleared"
        );

        let mut cpu = run_program(
            "
            SED
            LDA #$81
            ADC #$92
            STA $FF
            PHP
            ",
        );
        assert_eq!(
            cpu.read_memory_at_address(0xFF),
            0x73,
            "0x81 + 0x92 = 0x73 in BCD"
        );
        assert_eq!(
            cpu.read_memory_at_address(0x01FD) & 0x01,
            0x01,
            "0x81 + 0x92 sets carry flag"
        );
    }

    #[test]
    fn and() {
        let mut cpu = run_program(
            "
            LDA #$FF
            STA $FF
            LDA #$AA
            AND #$55
            STA $FF
            PHP
        ",
        );
        assert_eq!(
            cpu.read_memory_at_address(0xFF),
            0x00,
            "(0xAA & 0x55) = 0x00"
        );
        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag cleared");

        let mut cpu = run_program(
            "
            LDA #$FF
            AND #$80
            STA $FF
            PHP
        ",
        );
        assert_eq!(cpu.read_memory_at_address(0xFF), 0x80, "0xFF & 0x80 = 0x80");
        assert_eq!(cpu.read_memory_at_address(0x01FD) & 0x80, 0x80, "negative bit set");
        assert_eq!(cpu.read_memory_at_address(0x01FD) & 0x02, 0x00, "zero bit not set");
    }

    #[test]
    fn asl() {
        let mut cpu = run_program(
            "
        LDA #$FF
        ASL
        STA $FF
        PHP
        ",
        );
        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFE, "asl result correct");
        assert!(status & 0x80 == 0x80, "negative bit set");
        assert!(status & 0x02 == 0x00, "zero bit not set");
        assert!(status & 0x01 == 0x01, "carry bit set");
    }

    #[test]
    fn bcc() {
        let mut cpu = run_program(
            "
        LDA #$FE
        ADC #$03
        BCC $02
        LDA #$FF
        STA $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "branch not taken");

        let mut cpu = run_program(
            "
        LDA #$FE
        ADC #$01
        BCC $02
        LDA #$FA
        STA $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "branch taken");
    }

    #[test]
    fn bcs() {
        let mut cpu = run_program(
            "
        LDA #$FE
        ADC #$03
        BCS $02
        LDA #$FF
        STA $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0x01, "branch taken");

        let mut cpu = run_program(
            "
        LDA #$FE
        ADC #$01
        BCS $02
        LDA #$FA
        STA $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFA, "branch not taken");
    }

    #[test]
    fn beq() {
        let mut cpu = run_program(
            "
        SEC
        LDA #$FF
        SBC #$FF
        BEQ $02
        LDA #$FF
        STA $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0x00, "branch taken");

        let mut cpu = run_program(
            "
        SEC
        LDA #$FF
        SBC #$FE
        BEQ $02
        LDA #$FF
        STA $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "branch not taken");
    }

    #[test]
    fn bne() {
        let mut cpu = run_program(
            "
        SEC
        LDA #$FF
        SBC #$FF
        BNE $02
        LDA #$FF
        STA $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "branch not taken");

        let mut cpu = run_program(
            "
        SEC
        LDA #$FF
        SBC #$FE
        BNE $02
        LDA #$FF
        STA $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0x01, "branch taken");
    }

    #[test]
    fn bit()
    {
        let mut cpu = run_program("
        LDA #$AA
        STA $FF
        LDA #$55
        BIT $FF
        PHP
        ");

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(status & 0x60, 0x60, "overflow flag unset");
        assert_eq!(status & 0x20, 0x20, "zero flag set");
    }

    #[test]
    fn bmi()
    {
        let mut cpu = run_program("
        SEC
        LDA #$00
        SBC #$01
        BMI $02
        LDA #$00
        STA $FF
        ");

        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "branch taken");

        let mut cpu = run_program("
        SEC
        LDA #$01
        SBC #$01
        BMI $02
        LDA #$00
        STA $FF
        ");

        assert_eq!(cpu.read_memory_at_address(0xFF), 0x00, "branch not taken");
    }

    #[test]
    fn sbc() {
        let mut cpu = run_program(
            "
            LDA #$76
            SEC
            SBC #$05
            STA $FF
            PHP
        ",
        );
        assert_eq!(
            cpu.read_memory_at_address(0xFF),
            0x71,
            "0x76 - 0x05 = 0x71"
        );
        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x01, 0x01, "no borrow (carry set)");
        assert_eq!(status & 0x80, 0x00, "negative bit not set");
        assert_eq!(status & 0x02, 0x00, "zero bit not set");

        let mut cpu = run_program(
            "
            ADC #$05
            SEC
            SBC #$0A
            STA $FF
            PHP
        ",
        );
        assert_eq!(
            cpu.read_memory_at_address(0xFF),
            0xFB,
            "0x5 - 0xA = -0x5 (0xFB)"
        );
        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x01, 0x00, "borrow (carry not set)");
        assert_eq!(status & 0x80, 0x80, "negative bit set");
        assert_eq!(status & 0x02, 0x00, "zero bit not set");
    }

    #[test]
    fn sbc_bcd() {
        let mut cpu = run_program(
            "
            SED
            LDA #$92
            SEC
            SBC #$25
            STA $FF
            PHP
        ",
        );
        assert_eq!(cpu.read_memory_at_address(0xFF), 0x67);
        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative bit not set");
        assert_eq!(status & 0x02, 0x00, "zero bit not set");
        assert_eq!(status & 0x01, 0x01, "carry bit set");
        let mut cpu = run_program(
            "
            SED
            LDA #$25
            SEC
            SBC #$92
            STA $FF
            PHP
        ",
        );
        assert_eq!(cpu.read_memory_at_address(0xFF), 0x33);
        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative bit not set");
        assert_eq!(status & 0x02, 0x00, "zero bit not set");
        assert_eq!(status & 0x01, 0x00, "carry bit not set");
    }
}

mod assembler;
mod bus;
mod mos6502;
mod ram;
pub mod rp2a03;
