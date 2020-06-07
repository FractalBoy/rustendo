use crate::bus::{Bus, Connect};
use crate::mos6502::Mos6502;
use crate::ram::Ram;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Rp2a03 {
    mos6502: Mos6502,
    bus: Rc<RefCell<Bus>>,
}

impl Rp2a03 {
    /// Create a new RP2A03 microprocessor.
    ///
    /// Optionally, initialize ram with a slice.
    ///
    /// ```
    /// use rustendo_lib::rp2a03::Rp2a03;
    /// // Initialize without memory
    /// let mut rp2a03 = Rp2a03::new(None);
    /// // Initialize with memory
    /// let mut rp2a03 = Rp2a03::new(Some(&[0x0, 0x1, 0x2]));
    /// ```
    pub fn new(mem: Option<&[u8]>) -> Self {
        let mut ram = Box::new(Ram::new());
        let mut bus = Bus::new();

        if let Some(mem) = mem {
            ram.load_mem(&mem);
        }
        let ram = Rc::new(RefCell::new(ram as Box<dyn Connect>));

        bus.connect(&ram);

        let bus = Rc::new(RefCell::new(bus));

        Rp2a03 {
            mos6502: Mos6502::new(&bus),
            bus,
        }
    }

    /// Run the microprocessor for one clock cycle.
    /// Returns whether the current instruction is complete.
    ///
    /// Really, everything happens in the first clock
    /// cycle and the remaining time is spent doing
    /// nothing. The clock cycles are needed to emulate
    /// the microprocessor correctly.
    ///
    /// ```
    /// use rustendo_lib::rp2a03::Rp2a03;
    /// let mut rp2a03 = Rp2a03::new(None);
    /// // Run until one instruction completes.
    /// while rp2a03.clock() {}
    /// ```
    pub fn clock(&mut self) -> bool {
        self.mos6502.clock()
    }

    /// Only used for testing
    #[allow(dead_code)]
    fn read_memory_at_address(&mut self, address: u16) -> u8 {
        let old_address = self.bus.borrow().read_address();
        self.bus
            .borrow_mut()
            .write_address(((address & 0xFF00) >> 8) as u8, (address & 0x00FF) as u8);
        let mem = self.bus.borrow().read();
        self.bus
            .borrow_mut()
            .write_address(old_address.0, old_address.1);
        mem
    }
}

#[cfg(test)]
mod tests {
    use super::Rp2a03;
    use crate::assembler::{self, AssemblerError};

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
        assert_eq!(status & 0x40, 0x00, "overflow flag unset");
        assert_eq!(status & 0x02, 0x02, "zero flag set");
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
    fn brk() {
        let mut cpu = run_program(
            "
            SEC
            LDA #$AA
            SBC #$AA
            BRK
        ",
        );

        assert_eq!(
            cpu.read_memory_at_address(0x01FD),
            0x00,
            "address after BRK stored on stack"
        );
        assert_eq!(
            cpu.read_memory_at_address(0x01FC),
            0x07,
            "address after BRK stored on stack"
        );
        assert_eq!(
            cpu.read_memory_at_address(0x01FB) & 0x02,
            0x02,
            "zero flag stored on stack"
        );
        assert_eq!(
            cpu.read_memory_at_address(0x01FB) & 0x01,
            0x01,
            "carry flag stored on stack"
        );
    }

    #[test]
    fn bvc() {
        let mut cpu = run_program(
            "
            LDA #$FF
            ADC #$05 // Result is 0x04, overflow set
            BVC $02  // Branch should not be taken, execute next instruction
            LDA #$FF
            STA $FF  // Store 0xFF in $FF 
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "branch not taken");

        let mut cpu = run_program(
            "
            LDA #$01
            ADC #$05 // Result is 0x06, overflow not set
            BVC $02  // Branch should be taken, continue with STA $FF
            LDA #$FF
            STA $FF  // Store 0x06 in $FF 
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0x06, "branch taken");
    }

    #[test]
    fn bvs() {
        let mut cpu = run_program(
            "
            LDA #$FF
            ADC #$05 // Result is 0x04, overflow set
            BVS $02  // Branch should be taken, continue with STA $FF
            LDA #$FF
            STA $FF  // Store 0x04 in $FF 
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0x04, "branch taken");

        let mut cpu = run_program(
            "
            LDA #$01
            ADC #$05 // Result is 0x06, overflow not set
            BVS $02  // Branch should not be taken, continue with STA $FF
            LDA #$FF
            STA $FF  // Store 0xFF in $FF 
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "branch taken");
    }

    #[test]
    fn cmp() {
        let mut cpu = run_program(
            "
            LDA #$10
            CMP #$05
            PHP
            ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");

        let mut cpu = run_program(
            "
            LDA #$10
            CMP #$10
            PHP
            ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x02, "zero flag set");

        let mut cpu = run_program(
            "
            LDA #$10
            CMP #$11
            PHP
            ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");
    }

    #[test]
    fn cpx() {
        let mut cpu = run_program(
            "
            LDX #$10
            CPX #$05
            PHP
            ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");

        let mut cpu = run_program(
            "
            LDX #$10
            CPX #$10
            PHP
            ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x02, "zero flag set");

        let mut cpu = run_program(
            "
            LDX #$10
            CPX #$11
            PHP
            ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");
    }

    #[test]
    fn cpy() {
        let mut cpu = run_program(
            "
            LDY #$10
            CPY #$05
            PHP
            ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");

        let mut cpu = run_program(
            "
            LDY #$10
            CPY #$10
            PHP
            ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x02, "zero flag set");

        let mut cpu = run_program(
            "
            LDY #$10
            CPY #$11
            PHP
            ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");
    }

    #[test]
    fn dec() {
        let mut cpu = run_program(
            "
            LDA #$02
            STA $FF
            LDA #$01
            STA $FF
            DEC $FF
            PHP
        ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(cpu.read_memory_at_address(0xFF), 0x00, "correct result");

        let mut cpu = run_program(
            "
            LDA #$02
            STA $FF
            LDA #$00
            STA $FF
            DEC $FF
            PHP
        ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn dex() {
        let mut cpu = run_program(
            "
            LDX #$02
            STX $FF
            LDX #$01
            STX $FF
            DEX
            STX $FF
            PHP
        ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(cpu.read_memory_at_address(0xFF), 0x00, "correct result");

        let mut cpu = run_program(
            "
            LDX #$02
            STX $FF
            LDX #$00
            STX $FF
            DEX
            STX $FF
            PHP
        ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn dey() {
        let mut cpu = run_program(
            "
            LDY #$02
            STY $FF
            LDY #$01
            STY $FF
            DEY
            STY $FF
            PHP
        ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(cpu.read_memory_at_address(0xFF), 0x00, "correct result");

        let mut cpu = run_program(
            "
            LDY #$02
            STY $FF
            LDY #$00
            STY $FF
            DEY
            STY $FF
            PHP
        ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn eor() {
        let mut cpu = run_program(
            "
            LDA #$55
            EOR #$AA
            STA $FF
            PHP
        ",
        );

        assert_eq!(
            cpu.read_memory_at_address(0xFF),
            0xFF,
            "0x55 xor 0xAA = 0xFF"
        );
        assert_eq!(
            cpu.read_memory_at_address(0x01FD) & 0x80,
            0x80,
            "negative bit set"
        );

        let mut cpu = run_program(
            "
            LDA #$FF
            EOR #$FF
            STA $FF
            PHP
        ",
        );

        assert_eq!(
            cpu.read_memory_at_address(0xFF),
            0x00,
            "0xFF xor 0xFF = 0x00"
        );
        assert_eq!(
            cpu.read_memory_at_address(0x01FD) & 0x02,
            0x02,
            "zero bit set"
        );
    }

    #[test]
    fn inc() {
        let mut cpu = run_program(
            "
            LDA #$02
            STA $FF
            LDA #$FF
            STA $FF
            INC $FF
            PHP
        ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(cpu.read_memory_at_address(0xFF), 0x00, "correct result");

        let mut cpu = run_program(
            "
            LDA #$02
            STA $FF
            LDA #$FE
            STA $FF
            INC $FF
            PHP
        ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn inx() {
        let mut cpu = run_program(
            "
            LDX #$02
            STX $FF
            LDX #$FF
            STX $FF
            INX
            STX $FF
            PHP
        ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(cpu.read_memory_at_address(0xFF), 0x00, "correct result");

        let mut cpu = run_program(
            "
            LDX #$02
            STX $FF
            LDX #$FE
            STX $FF
            INX
            STX $FF
            PHP
        ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn iny() {
        let mut cpu = run_program(
            "
            LDY #$02
            STY $FF
            LDY #$FF
            STY $FF
            INY
            STY $FF
            PHP
        ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(cpu.read_memory_at_address(0xFF), 0x00, "correct result");

        let mut cpu = run_program(
            "
            LDY #$02
            STY $FF
            LDY #$FE
            STY $FF
            INY
            STY $FF
            PHP
        ",
        );

        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn jmp() {
        let mut cpu = run_program(
            "
            JMP $0800
            NOP
            NOP
            LDA #$FF // These two lines should not execute
            STA $FF  // so $FF should be empty.
            NOP
        ",
        );

        assert_eq!(
            cpu.read_memory_at_address(0xFF),
            0x00,
            "load and store jumped over"
        );
    }

    #[test]
    fn jsr() {
        let mut cpu = run_program(
            "
            JSR $0800
            LDA #$FF
            STA $FF
            LDA #$FF
            STA $FE
        ",
        );

        assert_ne!(
            cpu.read_memory_at_address(0xFF),
            0xFF,
            "first store skipped"
        );
        assert_ne!(
            cpu.read_memory_at_address(0xFE),
            0xFF,
            "second store skipped"
        );
        assert_eq!(cpu.read_memory_at_address(0x01FD), 0x00, "high byte = 0x00");
        assert_eq!(cpu.read_memory_at_address(0x01FC), 0x02, "low byte = 0x02");
    }

    #[test]
    fn lsr() {
        let mut cpu = run_program(
            "
        LDA #$FF
        STA $FF
        LSR $FF
        PHP
       ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0x7F, "0xFF >> 1 = 0x7F");
        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x01, 0x01, "carry bit set");
        assert_eq!(status & 0x02, 0x00, "zero bit unset");
        assert_eq!(status & 0x80, 0x00, "negative bit unset");

        let mut cpu = run_program(
            "
            LDA #$01
            STA $FF
            LSR $FF
            PHP
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0x00, "0x01 >> 1 = 0x00");
        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x01, 0x01, "carry bit set");
        assert_eq!(status & 0x02, 0x02, "zero bit set");
        assert_eq!(status & 0x80, 0x00, "negative bit unset");
    }

    #[test]
    fn ora() {
        let mut cpu = run_program(
            "
            LDA #$AA
            STA $FF
            LDA #$55
            ORA $FF
            STA $FF
            PHP
        ",
        );

        assert_eq!(cpu.read_memory_at_address(0xFF), 0xFF, "0xAA | 0x55 = 0xFF");
        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x80, "result negative");
        assert_eq!(status & 0x02, 0x00, "result not zero");

        let mut cpu = run_program(
            "
            LDA #$00
            STA $FF
            LDA #$00
            ORA $FF
            STA $FF
            PHP
        ",
        );
        assert_eq!(cpu.read_memory_at_address(0xFF), 0x00, "0x00 | 0x00 = 0x00");
        let status = cpu.read_memory_at_address(0x01FD);
        assert_eq!(status & 0x80, 0x00, "result not negative");
        assert_eq!(status & 0x02, 0x02, "result zero");
    }

    #[test]
    fn pha() {
        let mut cpu = run_program(
            "
            LDA #$FF
            PHA
        ",
        );

        assert_eq!(
            cpu.read_memory_at_address(0x01FD),
            0xFF,
            "accumulator pushed on stack"
        );
    }

    #[test]
    fn pla() {
        let mut cpu = run_program(
            "
        LDA #$FF
        PHA
        LDA #$00
        PLA
        STA $FF
        ",
        );

        assert_eq!(
            cpu.read_memory_at_address(0x01FD),
            0xFF,
            "accumulator pulled from stack"
        );
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
