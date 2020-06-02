#[cfg(test)]
mod tests {
    use super::mos6502::AddressingMode;
    use super::rp2a03::Rp2a03;
    extern crate regex;

    #[test]
    fn adc_no_carry() {
        let mut cpu = run_program(
            "
        LDA #$01
        ADC #$01
        STA $0D
        LDA #$00
        ADC #$00
        STA $0E
        ",
        );
        assert_eq!(cpu.read_memory_at_address(0xD), 2, "0x1 + 0x1 = 0x2");
        assert_eq!(cpu.read_memory_at_address(0xE), 0, "carry bit cleared");
    }

    #[test]
    fn adc_with_carry() {
        let mut cpu = run_program(
            "
            LDA #$FF
            ADC #$FF
            STA $0D
            LDA #$00
            ADC #$00
            STA $0E
        ",
        );
        assert_eq!(cpu.read_memory_at_address(0xD), 0xFE, "0xFF + 0xFF = 0xFE");
        assert_eq!(
            cpu.read_memory_at_address(0xE),
            0x1,
            "0xFF + 0xFF sets carry flag"
        );
    }

    #[test]
    fn adc_bcd() {
        let mut cpu = run_program(
            "
            SED
            LDA #$10
            ADC #$10
            STA $0D
            LDA #$00
            ADC #$00
            STA $0E
        ",
        );
        assert_eq!(
            cpu.read_memory_at_address(0xD),
            0x20,
            "0x10 + 0x10 = 0x20 in BCD"
        );
        assert_eq!(cpu.read_memory_at_address(0xE), 0x0, "carry bit cleared");
        let mut cpu = run_program(
            "
            SED
            LDA #$81
            ADC #$92
            STA $0D
            LDA #$00
            ADC #$00
            STA $0E
            ",
        );
        assert_eq!(
            cpu.read_memory_at_address(0xD),
            0x73,
            "0x81 + 0x92 = 0x73 in BCD"
        );
        assert_eq!(
            cpu.read_memory_at_address(0xE),
            0x1,
            "0x81 + 0x92 sets carry flag"
        );
    }

    #[test]
    fn and_eq_zero() {
        let mut cpu = run_program(
            "
            ADC #$FF
            AND #$00
            BMI $02
            BEQ $02
            ADC #$02
            ADC #$01
            STA $0D
        ",
        );
        assert_eq!(
            cpu.read_memory_at_address(0xD),
            0x1,
            "((0xFF & 0xFF) + 0x01)= 0x01"
        );
    }

    #[test]
    fn and_eq_negative() {
        let mut cpu = run_program(
            "
            LDA #$FF
            AND #$80
            BEQ $02
            BMI $02
            ADC #$02
            ADC #$01
            STA $0D
        ",
        );
        assert_eq!(
            cpu.read_memory_at_address(0xD),
            0x81,
            "(0xFF & 0x80) + 0x01 = 0x81"
        );
    }

    #[test]
    fn sbc_without_borrow() {
        let mut cpu = run_program(
            "
            LDA #$76
            SEC
            SBC #$05
            BMI $08
            STA $0F
            LDA #$00
            ADC #$00
            STA $10
        ",
        );
        assert_eq!(
            cpu.read_memory_at_address(0xF),
            0x71,
            "0x76 - 0x05 = 0x71, BMI branch not taken"
        );
        assert_eq!(
            cpu.read_memory_at_address(0x10),
            0x1,
            "no borrow (carry set)"
        );
    }

    #[test]
    fn sbc_with_borrow() {
        let mut cpu = run_program(
            "
            ADC #$05
            SEC
            SBC #$0A
            BPL $09
            STA $0F
            LDA #$00
            ADC #$00
            STA $10
        ",
        );
        assert_eq!(
            cpu.read_memory_at_address(0xF),
            0xFB,
            "0x5 - 0xA = -0x5 (0xFB), BPL branch not taken"
        );
        assert_eq!(
            cpu.read_memory_at_address(0x10),
            0x0,
            "borrow (carry not set)"
        );
    }

    #[test]
    fn sbc_bcd() {
        let mut cpu = run_program(
            "
            SED
            LDA #$92
            SEC
            SBC #$25
            BMI #$08
            STA $10
            LDA #$00
            ADC #$00
            STA $11
        ",
        );
        assert_eq!(cpu.read_memory_at_address(0x10), 0x67);
        assert_eq!(cpu.read_memory_at_address(0x11), 0x0);
        let mut cpu = run_program(
            "
            SED
            LDA #$25
            SEC
            SBC #$92
            BMI #$08
            STA $10
            LDA #$00
            ADC #$00
            STA $11
        ",
        );
        assert_eq!(cpu.read_memory_at_address(0x10), 0x33);
        assert_eq!(
            cpu.read_memory_at_address(0x11),
            0x0,
            "borrow set (carry unset)"
        );
    }

    fn assemble_program(program: &str) -> Vec<Vec<u8>> {
        let lines: Vec<&str> = program.split("\n").collect();
        let mut program: Vec<Vec<u8>> = vec![];

        for line in lines {
            let whitespace_re = regex::Regex::new("^\\s+|\\s+$").unwrap();
            let line = whitespace_re.replace_all(line, "");
            let fields: Vec<&str> = line.split_whitespace().collect();

            if fields.len() == 0 {
                continue;
            }
            println!("{:?}", fields);

            if fields.len() == 1 {
                let instruction = fields[0];

                if let Some(byte) = lookup_instruction(instruction, AddressingMode::Implied) {
                    program.push(vec![byte]);
                    continue;
                } else if let Some(byte) =
                    lookup_instruction(instruction, AddressingMode::Accumulator)
                {
                    program.push(vec![byte]);
                    continue;
                }
            } else {
                let instruction = fields[0];
                let parameter = fields[1];

                let immediate_re = regex::Regex::new("#\\$([A-F\\d]{2})").unwrap();
                let zero_page_re = regex::Regex::new("\\$([A-F\\d]{2})").unwrap();
                let zero_page_x_re = regex::Regex::new("\\$([A-F\\d]{2})\\s*,\\s*[Xx]").unwrap();
                let zero_page_y_re = regex::Regex::new("\\$([A-F\\d{2}])\\s*,\\s*[Yy]").unwrap();
                let absolute_re = regex::Regex::new("\\$([A-F\\d]{4})").unwrap();
                let absolute_x_re = regex::Regex::new("\\$([A-F\\d]{4})\\s*,\\s*[Xx]").unwrap();
                let absolute_y_re = regex::Regex::new("\\$([A-F\\d]{4})\\s*,\\s*[Yy]").unwrap();
                let indirect_re = regex::Regex::new("\\(\\$([A-F\\d]{4})\\)").unwrap();
                let indirect_x_re =
                    regex::Regex::new("\\(\\$([A-F\\d]{4})\\s*,\\s*[Xx]\\)").unwrap();
                let indirect_y_re =
                    regex::Regex::new("\\(\\$([A-F\\d]{4})\\)\\s*,\\s*[Yy]").unwrap();

                if let Some(captures) = immediate_re.captures(parameter) {
                    if let Some(value) = captures.get(1) {
                        let value = value.as_str();
                        let value = u8::from_str_radix(&value, 16).unwrap();
                        if let Some(byte) =
                            lookup_instruction(instruction, AddressingMode::Immediate)
                        {
                            program.push(vec![byte, value]);
                            continue;
                        }
                    }
                } else if let Some(captures) = zero_page_re.captures(parameter) {
                    if let Some(address) = captures.get(1) {
                        let address = address.as_str();
                        let address = u8::from_str_radix(&address, 16).unwrap();
                        if let Some(byte) =
                            lookup_instruction(instruction, AddressingMode::ZeroPage)
                        {
                            program.push(vec![byte, address]);
                            continue;
                        } else if let Some(byte) =
                            lookup_instruction(instruction, AddressingMode::Relative)
                        {
                            program.push(vec![byte, address]);
                            continue;
                        }
                    }
                } else if let Some(captures) = zero_page_x_re.captures(parameter) {
                    if let Some(address) = captures.get(1) {
                        let address = address.as_str();
                        let address = u8::from_str_radix(&address, 16).unwrap();
                        if let Some(byte) =
                            lookup_instruction(instruction, AddressingMode::ZeroPageX)
                        {
                            program.push(vec![byte, address]);
                            continue;
                        }
                    }
                } else if let Some(captures) = zero_page_y_re.captures(parameter) {
                    if let Some(address) = captures.get(1) {
                        let address = address.as_str();
                        let address = u8::from_str_radix(&address, 16).unwrap();
                        if let Some(byte) =
                            lookup_instruction(instruction, AddressingMode::ZeroPageY)
                        {
                            program.push(vec![byte, address]);
                            continue;
                        }
                    }
                } else if let Some(captures) = absolute_re.captures(parameter) {
                    if let Some(address) = captures.get(1) {
                        let address = address.as_str();
                        let address = u16::from_str_radix(&address, 16).unwrap();
                        let [address_high, address_low] = address.to_be_bytes();
                        if let Some(byte) =
                            lookup_instruction(instruction, AddressingMode::Absolute)
                        {
                            program.push(vec![byte, address_low, address_high]);
                            continue;
                        }
                    }
                } else if let Some(captures) = absolute_x_re.captures(parameter) {
                    if let Some(address) = captures.get(1) {
                        let address = address.as_str();
                        let address = u16::from_str_radix(&address, 16).unwrap();
                        let [address_high, address_low] = address.to_be_bytes();
                        if let Some(byte) =
                            lookup_instruction(instruction, AddressingMode::AbsoluteX)
                        {
                            program.push(vec![byte, address_low, address_high]);
                            continue;
                        }
                    }
                } else if let Some(captures) = absolute_y_re.captures(parameter) {
                    if let Some(address) = captures.get(1) {
                        let address = address.as_str();
                        let address = u16::from_str_radix(&address, 16).unwrap();
                        let [address_high, address_low] = address.to_be_bytes();
                        if let Some(byte) =
                            lookup_instruction(instruction, AddressingMode::AbsoluteY)
                        {
                            program.push(vec![byte, address_low, address_high]);
                            continue;
                        }
                    }
                } else if let Some(captures) = indirect_re.captures(parameter) {
                    if let Some(address) = captures.get(1) {
                        let address = address.as_str();
                        let address = u8::from_str_radix(&address, 16).unwrap();
                        if let Some(byte) =
                            lookup_instruction(instruction, AddressingMode::Indirect)
                        {
                            program.push(vec![byte, address]);
                            continue;
                        }
                    }
                } else if let Some(captures) = indirect_x_re.captures(parameter) {
                    if let Some(address) = captures.get(1) {
                        let address = address.as_str();
                        let address = u8::from_str_radix(&address, 16).unwrap();
                        if let Some(byte) =
                            lookup_instruction(instruction, AddressingMode::IndirectX)
                        {
                            program.push(vec![byte, address]);
                            continue;
                        }
                    }
                } else if let Some(captures) = indirect_y_re.captures(parameter) {
                    if let Some(address) = captures.get(1) {
                        let address = address.as_str();
                        let address = u8::from_str_radix(&address, 16).unwrap();
                        if let Some(byte) =
                            lookup_instruction(instruction, AddressingMode::IndirectY)
                        {
                            program.push(vec![byte, address]);
                            continue;
                        }
                    }
                }
            }
        }

        program
    }

    fn run_program(program: &str) -> Rp2a03 {
        let program = assemble_program(&program);
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

    fn lookup_instruction(instruction: &str, addressing_mode: AddressingMode) -> Option<u8> {
        println!(
            "looking up instruction {} with addressing mode {:?}",
            instruction, addressing_mode
        );
        match instruction {
            "ADC" => match addressing_mode {
                AddressingMode::Immediate => Some(0x69),
                AddressingMode::ZeroPage => Some(0x65),
                AddressingMode::ZeroPageX => Some(0x75),
                AddressingMode::Absolute => Some(0x6D),
                AddressingMode::AbsoluteX => Some(0x7D),
                AddressingMode::AbsoluteY => Some(0x79),
                AddressingMode::IndirectX => Some(0x61),
                AddressingMode::IndirectY => Some(0x71),
                _ => None,
            },
            "AND" => match addressing_mode {
                AddressingMode::Immediate => Some(0x29),
                AddressingMode::ZeroPage => Some(0x25),
                AddressingMode::ZeroPageX => Some(0x35),
                AddressingMode::Absolute => Some(0x2D),
                AddressingMode::AbsoluteX => Some(0x3D),
                AddressingMode::AbsoluteY => Some(0x39),
                AddressingMode::IndirectX => Some(0x21),
                AddressingMode::IndirectY => Some(0x31),
                _ => None,
            },
            "ASL" => match addressing_mode {
                AddressingMode::Accumulator => Some(0x0A),
                AddressingMode::ZeroPage => Some(0x06),
                AddressingMode::ZeroPageX => Some(0x16),
                AddressingMode::Absolute => Some(0x0E),
                AddressingMode::AbsoluteX => Some(0x1E),
                _ => None,
            },
            "BCC" => match addressing_mode {
                AddressingMode::Relative => Some(0x90),
                _ => None,
            },
            "BCS" => match addressing_mode {
                AddressingMode::Relative => Some(0xB0),
                _ => None,
            },
            "BEQ" => match addressing_mode {
                AddressingMode::Relative => Some(0xF0),
                _ => None,
            },
            "BIT" => match addressing_mode {
                AddressingMode::ZeroPage => Some(0x24),
                AddressingMode::Absolute => Some(0x2C),
                _ => None,
            },
            "BMI" => match addressing_mode {
                AddressingMode::Relative => Some(0x30),
                _ => None,
            },
            "BNE" => match addressing_mode {
                AddressingMode::Relative => Some(0xD0),
                _ => None,
            },
            "BPL" => match addressing_mode {
                AddressingMode::Relative => Some(0x10),
                _ => None,
            },
            "BRK" => match addressing_mode {
                AddressingMode::Implied => Some(0x00),
                _ => None,
            },
            "BVC" => match addressing_mode {
                AddressingMode::Relative => Some(0x50),
                _ => None,
            },
            "BVS" => match addressing_mode {
                AddressingMode::Relative => Some(0x70),
                _ => None,
            },
            "CLC" => match addressing_mode {
                AddressingMode::Implied => Some(0x18),
                _ => None,
            },
            "CLD" => match addressing_mode {
                AddressingMode::Implied => Some(0xD8),
                _ => None,
            },
            "CLI" => match addressing_mode {
                AddressingMode::Implied => Some(0x58),
                _ => None,
            },
            "CLV" => match addressing_mode {
                AddressingMode::Implied => Some(0xB8),
                _ => None,
            },
            "CMP" => match addressing_mode {
                AddressingMode::Immediate => Some(0xC9),
                AddressingMode::ZeroPage => Some(0xC5),
                AddressingMode::ZeroPageX => Some(0xD5),
                AddressingMode::Absolute => Some(0xCD),
                AddressingMode::AbsoluteX => Some(0xDD),
                AddressingMode::AbsoluteY => Some(0xD9),
                AddressingMode::IndirectX => Some(0xC1),
                AddressingMode::IndirectY => Some(0xD1),
                _ => None,
            },
            "CPX" => match addressing_mode {
                AddressingMode::Immediate => Some(0xE0),
                AddressingMode::ZeroPage => Some(0xE4),
                AddressingMode::Absolute => Some(0xEC),
                _ => None,
            },
            "CPY" => match addressing_mode {
                AddressingMode::Immediate => Some(0xC0),
                AddressingMode::ZeroPage => Some(0xC4),
                AddressingMode::Absolute => Some(0xCC),
                _ => None,
            },
            "DEC" => match addressing_mode {
                AddressingMode::ZeroPage => Some(0xC6),
                AddressingMode::ZeroPageX => Some(0xD6),
                AddressingMode::Absolute => Some(0xCE),
                AddressingMode::AbsoluteX => Some(0xDE),
                _ => None,
            },
            "DEX" => match addressing_mode {
                AddressingMode::Implied => Some(0xCA),
                _ => None,
            },
            "DEY" => match addressing_mode {
                AddressingMode::Implied => Some(0x88),
                _ => None,
            },
            "EOR" => match addressing_mode {
                AddressingMode::Immediate => Some(0x49),
                AddressingMode::ZeroPage => Some(0x45),
                AddressingMode::ZeroPageX => Some(0x55),
                AddressingMode::Absolute => Some(0x4D),
                AddressingMode::AbsoluteX => Some(0x5D),
                AddressingMode::AbsoluteY => Some(0x59),
                AddressingMode::IndirectX => Some(0x41),
                AddressingMode::IndirectY => Some(0x51),
                _ => None,
            },
            "INC" => match addressing_mode {
                AddressingMode::ZeroPage => Some(0xE6),
                AddressingMode::ZeroPageX => Some(0xF6),
                AddressingMode::Absolute => Some(0xEE),
                AddressingMode::AbsoluteX => Some(0xFE),
                _ => None,
            },
            "INX" => match addressing_mode {
                AddressingMode::Implied => Some(0xE8),
                _ => None,
            },
            "INY" => match addressing_mode {
                AddressingMode::Implied => Some(0xC8),
                _ => None,
            },
            "JMP" => match addressing_mode {
                AddressingMode::Absolute => Some(0x46),
                AddressingMode::Indirect => Some(0x6C),
                _ => None,
            },
            "JSR" => match addressing_mode {
                AddressingMode::Absolute => Some(0x20),
                _ => None,
            },
            "LDA" => match addressing_mode {
                AddressingMode::Immediate => Some(0xA9),
                AddressingMode::ZeroPage => Some(0xA5),
                AddressingMode::ZeroPageX => Some(0xB5),
                AddressingMode::Absolute => Some(0xAD),
                AddressingMode::AbsoluteX => Some(0xBD),
                AddressingMode::AbsoluteY => Some(0xB9),
                AddressingMode::IndirectX => Some(0xA1),
                AddressingMode::IndirectY => Some(0xB1),
                _ => None,
            },
            "LDX" => match addressing_mode {
                AddressingMode::Immediate => Some(0xA2),
                AddressingMode::ZeroPage => Some(0xA6),
                AddressingMode::ZeroPageY => Some(0xB6),
                AddressingMode::Absolute => Some(0xAE),
                AddressingMode::AbsoluteY => Some(0xBE),
                _ => None,
            },
            "LDY" => match addressing_mode {
                AddressingMode::Immediate => Some(0xA0),
                AddressingMode::ZeroPage => Some(0xA4),
                AddressingMode::ZeroPageX => Some(0xB4),
                AddressingMode::Absolute => Some(0xAC),
                AddressingMode::AbsoluteX => Some(0xBC),
                _ => None,
            },
            "LSR" => match addressing_mode {
                AddressingMode::Accumulator => Some(0x4A),
                AddressingMode::ZeroPage => Some(0x46),
                AddressingMode::ZeroPageX => Some(0x56),
                AddressingMode::Absolute => Some(0x4E),
                AddressingMode::AbsoluteX => Some(0x5E),
                _ => None,
            },
            "NOP" => match addressing_mode {
                AddressingMode::Implied => Some(0xEA),
                _ => None,
            },
            "ORA" => match addressing_mode {
                AddressingMode::Immediate => Some(0x09),
                AddressingMode::ZeroPage => Some(0x05),
                AddressingMode::ZeroPageX => Some(0x15),
                AddressingMode::Absolute => Some(0x0D),
                AddressingMode::AbsoluteX => Some(0x1D),
                AddressingMode::AbsoluteY => Some(0x19),
                AddressingMode::IndirectX => Some(0x01),
                AddressingMode::IndirectY => Some(0x11),
                _ => None,
            },
            "PHA" => match addressing_mode {
                AddressingMode::Implied => Some(0x48),
                _ => None,
            },
            "PHP" => match addressing_mode {
                AddressingMode::Implied => Some(0x08),
                _ => None,
            },
            "PLA" => match addressing_mode {
                AddressingMode::Implied => Some(0x68),
                _ => None,
            },
            "PLP" => match addressing_mode {
                AddressingMode::Implied => Some(0x28),
                _ => None,
            },
            "ROL" => match addressing_mode {
                AddressingMode::Accumulator => Some(0x2A),
                AddressingMode::ZeroPage => Some(0x26),
                AddressingMode::ZeroPageX => Some(0x36),
                AddressingMode::Absolute => Some(0x2E),
                AddressingMode::AbsoluteX => Some(0x3E),
                _ => None,
            },
            "ROR" => match addressing_mode {
                AddressingMode::Accumulator => Some(0x6A),
                AddressingMode::ZeroPage => Some(0x66),
                AddressingMode::ZeroPageX => Some(0x76),
                AddressingMode::Absolute => Some(0x6E),
                AddressingMode::AbsoluteX => Some(0x7E),
                _ => None,
            },
            "RTI" => match addressing_mode {
                AddressingMode::Implied => Some(0x40),
                _ => None,
            },
            "RTS" => match addressing_mode {
                AddressingMode::Implied => Some(0x60),
                _ => None,
            },
            "SBC" => match addressing_mode {
                AddressingMode::Immediate => Some(0xE9),
                AddressingMode::ZeroPage => Some(0xE5),
                AddressingMode::ZeroPageX => Some(0xF5),
                AddressingMode::Absolute => Some(0xED),
                AddressingMode::AbsoluteX => Some(0xFD),
                AddressingMode::AbsoluteY => Some(0xF9),
                AddressingMode::IndirectX => Some(0xE1),
                AddressingMode::IndirectY => Some(0xF1),
                _ => None,
            },
            "SEC" => match addressing_mode {
                AddressingMode::Implied => Some(0x38),
                _ => None,
            },
            "SED" => match addressing_mode {
                AddressingMode::Implied => Some(0xF8),
                _ => None,
            },
            "SEI" => match addressing_mode {
                AddressingMode::Implied => Some(0x78),
                _ => None,
            },
            "STA" => match addressing_mode {
                AddressingMode::ZeroPage => Some(0x85),
                AddressingMode::ZeroPageX => Some(0x95),
                AddressingMode::Absolute => Some(0x8D),
                AddressingMode::AbsoluteX => Some(0x9D),
                AddressingMode::AbsoluteY => Some(0x99),
                AddressingMode::IndirectX => Some(0x81),
                AddressingMode::IndirectY => Some(0x91),
                _ => None,
            },
            "STX" => match addressing_mode {
                AddressingMode::ZeroPage => Some(0x86),
                AddressingMode::ZeroPageY => Some(0x96),
                AddressingMode::Absolute => Some(0x8E),
                _ => None,
            },
            "STY" => match addressing_mode {
                AddressingMode::ZeroPage => Some(0x84),
                AddressingMode::ZeroPageX => Some(0x94),
                AddressingMode::Absolute => Some(0x8C),
                _ => None,
            },
            "TAX" => match addressing_mode {
                AddressingMode::Implied => Some(0xAA),
                _ => None,
            },
            "TAY" => match addressing_mode {
                AddressingMode::Implied => Some(0xA8),
                _ => None,
            },
            "TSX" => match addressing_mode {
                AddressingMode::Implied => Some(0xBA),
                _ => None,
            },
            "TXA" => match addressing_mode {
                AddressingMode::Implied => Some(0x8A),
                _ => None,
            },
            "TXS" => match addressing_mode {
                AddressingMode::Implied => Some(0x9A),
                _ => None,
            },
            "TYA" => match addressing_mode {
                AddressingMode::Implied => Some(0x98),
                _ => None,
            },
            _ => None,
        }
    }
}

mod bus;
mod mos6502;
mod ram;
pub mod rp2a03;
