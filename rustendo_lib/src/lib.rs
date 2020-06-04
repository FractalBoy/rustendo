#[cfg(test)]
mod tests {
    use super::assembler::{self, AssemblerError};
    use super::rp2a03::Rp2a03;

    fn run_program(program: &str) -> Rp2a03 {
        match assembler::run_program(program) {
            Ok(cpu) => return cpu,
            Err(error) => {
                match error {
                    AssemblerError::InvalidAddress(line) => {
                        panic!("Invalid address at line {}", line)
                    }
                    AssemblerError::InvalidAddressingMode(line) => {
                        panic!("Invalid addressing mode at line {}", line)
                    }
                    AssemblerError::InvalidInstruction(line) => {
                        panic!("Invalid instruction at line {}", line)
                    }
                    AssemblerError::InvalidValue(line) => {
                        panic!("Invalid immediate value at line {}", line)
                    }
                };
            }
        }
    }

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
        assert_eq!(
            cpu.read_memory_at_address(0x01FD) & 0x80,
            0x80,
            "negative bit set"
        );
        assert_eq!(
            cpu.read_memory_at_address(0x01FD) & 0x02,
            0x00,
            "zero bit not set"
        );
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
        ADC #$03 // Result is 0x101, carry set
        BCC $02  // Branch should not be taken, next line executes
        LDA #$FF
        STA $FF  // 0xFF stored to $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "branch not taken");

        let mut cpu = run_program(
            "
        LDA #$FE
        ADC #$01 // Result is 0xFF, carry cleared
        BCC $02  // Branch should be taken to STA $FF
        LDA #$FA
        STA $FF  // 0xFF stored to $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "branch taken");
    }

    #[test]
    fn bcs() {
        let mut cpu = run_program(
            "
        LDA #$FE
        ADC #$03 // Result is 0x101, carry set
        BCS $02  // Branch should be taken to STA $FF
        LDA #$FF
        STA $FF  // 0x01 stored to $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0x01, "branch taken");

        let mut cpu = run_program(
            "
        LDA #$FE
        ADC #$01 // Result is 0xFF, carry cleared
        BCS $02  // Branch should not be taken, next line executes
        LDA #$FA
        STA $FF  // 0xFA stored to $FF
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
        SBC #$FF // Result is 0x00, zero set
        BEQ $02  // Branch should be taken to STA $FF
        LDA #$FF
        STA $FF  // 0x00 stored to $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0x00, "branch taken");

        let mut cpu = run_program(
            "
        SEC
        LDA #$FF
        SBC #$FE // Result is 0x01, zero cleared
        BEQ $02  // Result should not be taken, next line executes
        LDA #$FF
        STA $FF  // 0xFF stored to $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "branch not taken");
    }

    #[test]
    fn bmi() {
        let mut cpu = run_program(
            "
        SEC
        LDA #$00
        SBC #$01 // Result is 0xFF, negative bit set
        BMI $02  // Branch should be taken to STA $FF
        LDA #$01
        STA $FF  // 0xFF stored to $FF 
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "branch taken");

        let mut cpu = run_program(
            "
        SEC
        LDA #$01
        SBC #$01 // Result is 0x00, negative bit not set
        BMI $02  // Branch should not be taken, next line executes
        LDA #$02
        STA $FF  // 0x02 stored to $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0x02, "branch not taken");
    }
    
    #[test]
    fn bpl() {
        let mut cpu = run_program(
            "
        SEC
        LDA #$00
        SBC #$01 // Result is 0xFF, negative bit set
        BPL $02  // Branch should not be taken to STA $FF
        LDA #$01
        STA $FF  // 0xFF stored to $FF 
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0x01, "branch taken");

        let mut cpu = run_program(
            "
        SEC
        LDA #$04
        SBC #$01 // Result is 0x00, negative bit not set
        BPL $02  // Branch should be taken, next line executes
        LDA #$02
        STA $FF  // 0x03 stored to $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0x03, "branch not taken");
    }

    #[test]
    fn brk()
    {
        let mut cpu = run_program("
            SEC
            LDA #$AA
            SBC #$AA
            BRK
        ");

        assert_eq!(cpu.read_memory_at_address(0x01FD), 0x00, "address after BRK stored on stack");
        assert_eq!(cpu.read_memory_at_address(0x01FC), 0x07, "address after BRK stored on stack");
        assert_eq!(cpu.read_memory_at_address(0x01FB) & 0x02, 0x02, "zero flag stored on stack");
        assert_eq!(cpu.read_memory_at_address(0x01FB) & 0x01, 0x01, "carry flag stored on stack");
    }

    #[test]
    fn bne() {
        let mut cpu = run_program(
            "
        SEC
        LDA #$FF
        SBC #$FF // Result is 0x00, zero set
        BNE $02  // Branch should not be taken, next line executes
        LDA #$FF
        STA $FF  // 0xFF stored to $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "branch not taken");

        let mut cpu = run_program(
            "
        SEC
        LDA #$FF
        SBC #$FE // Result is 0x01, zero cleared
        BNE $02  // Branch should be taken to STA $FF
        LDA #$FF
        STA $FF  // 0x01 stored to $FF
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0x01, "branch taken");
    }

    #[test]
    fn bit() {
        let mut cpu = run_program(
            "
        LDA #$AA
        STA $FF
        LDA #$55
        BIT $FF
        PHP
        ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(status & 0x60, 0x60, "overflow flag unset");
        assert_eq!(status & 0x20, 0x20, "zero flag set");
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
        assert_eq!(cpu.read_memory_at_address(0xFF), 0x71, "0x76 - 0x05 = 0x71");
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
