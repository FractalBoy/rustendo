use crate::cpu_bus::Bus;
use crate::mos6502::{AddressingMode, Mos6502};
use regex::Regex;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub enum AssemblerError {
    InvalidInstruction(u32),
    InvalidAddressingMode(u32),
    InvalidValue(u32),
    InvalidAddress(u32),
}

pub fn assemble_program(program: &str) -> Result<Vec<Vec<u8>>, AssemblerError> {
    let immediate_re: Regex = Regex::new("#\\$([A-F\\d]{2})$").unwrap();
    let zero_page_re: Regex = Regex::new("\\$([A-F\\d]{2})$").unwrap();
    let zero_page_x_re: Regex = Regex::new("\\$([A-F\\d]{2})\\s*,\\s*[Xx]$").unwrap();
    let zero_page_y_re: Regex = Regex::new("\\$([A-F\\d{2}])\\s*,\\s*[Yy]$").unwrap();
    let absolute_re: Regex = Regex::new("\\$([A-F\\d]{4})$").unwrap();
    let absolute_x_re: Regex = Regex::new("\\$([A-F\\d]{4})\\s*,\\s*[Xx]$").unwrap();
    let absolute_y_re: Regex = Regex::new("\\$([A-F\\d]{4})\\s*,\\s*[Yy]$").unwrap();
    let indirect_re: Regex = Regex::new("\\(\\$([A-F\\d]{4})\\)$").unwrap();
    let indirect_x_re: Regex = Regex::new("\\(\\$([A-F\\d]{4})\\s*,\\s*[Xx]\\)$").unwrap();
    let indirect_y_re: Regex = Regex::new("\\(\\$([A-F\\d]{4})\\)\\s*,\\s*[Yy]$").unwrap();
    let whitespace_re: Regex = Regex::new("^\\s+|\\s+$").unwrap();
    let comment_re: Regex = Regex::new("\\s*//.*$").unwrap();

    let lines: Vec<&str> = program.split("\n").collect();
    let mut program: Vec<Vec<u8>> = vec![];

    let mut line_number = 0;
    for line in lines {
        line_number += 1;
        let line = match whitespace_re.replace_all(line, "") {
            Cow::Owned(line) => line,
            Cow::Borrowed(line) => line.to_string(),
        };
        // Remove comments
        let line = comment_re.replace_all(line.as_str(), "");
        let fields: Vec<&str> = line.split_whitespace().collect();

        if fields.len() == 0 {
            continue;
        }

        if fields.len() == 1 {
            let instruction = fields[0];

            match lookup_instruction(instruction, AddressingMode::Implied) {
                Some(byte) => {
                    program.push(vec![byte]);
                    continue;
                }
                None => match lookup_instruction(instruction, AddressingMode::Accumulator) {
                    Some(byte) => {
                        program.push(vec![byte]);
                        continue;
                    }
                    None => return Err(AssemblerError::InvalidInstruction(line_number)),
                },
            }
        } else {
            let instruction = fields[0];
            let parameter = fields[1];

            if let Some(captures) = immediate_re.captures(parameter) {
                if let Some(value) = captures.get(1) {
                    let value = value.as_str();
                    let value = match u8::from_str_radix(&value, 16) {
                        Ok(value) => value,
                        Err(_) => return Err(AssemblerError::InvalidValue(line_number)),
                    };
                    match lookup_instruction(instruction, AddressingMode::Immediate) {
                        Some(byte) => {
                            program.push(vec![byte, value]);
                            continue;
                        }
                        None => return Err(AssemblerError::InvalidInstruction(line_number)),
                    }
                }
            } else if let Some(captures) = zero_page_re.captures(parameter) {
                if let Some(address) = captures.get(1) {
                    let address = address.as_str();
                    let address = match u8::from_str_radix(&address, 16) {
                        Ok(address) => address,
                        Err(_) => return Err(AssemblerError::InvalidAddress(line_number)),
                    };
                    match lookup_instruction(instruction, AddressingMode::ZeroPage) {
                        Some(byte) => {
                            program.push(vec![byte, address]);
                            continue;
                        }
                        None => match lookup_instruction(instruction, AddressingMode::Relative) {
                            Some(byte) => {
                                program.push(vec![byte, address]);
                                continue;
                            }
                            None => return Err(AssemblerError::InvalidInstruction(line_number)),
                        },
                    }
                }
            } else if let Some(captures) = zero_page_x_re.captures(parameter) {
                if let Some(address) = captures.get(1) {
                    let address = address.as_str();
                    let address = match u8::from_str_radix(&address, 16) {
                        Ok(address) => address,
                        Err(_) => return Err(AssemblerError::InvalidAddress(line_number)),
                    };
                    match lookup_instruction(instruction, AddressingMode::ZeroPageX) {
                        Some(byte) => {
                            program.push(vec![byte, address]);
                            continue;
                        }
                        None => return Err(AssemblerError::InvalidInstruction(line_number)),
                    }
                }
            } else if let Some(captures) = zero_page_y_re.captures(parameter) {
                if let Some(address) = captures.get(1) {
                    let address = address.as_str();
                    let address = match u8::from_str_radix(&address, 16) {
                        Ok(address) => address,
                        Err(_) => return Err(AssemblerError::InvalidAddress(line_number)),
                    };
                    match lookup_instruction(instruction, AddressingMode::ZeroPageY) {
                        Some(byte) => {
                            program.push(vec![byte, address]);
                            continue;
                        }
                        None => return Err(AssemblerError::InvalidInstruction(line_number)),
                    }
                }
            } else if let Some(captures) = absolute_re.captures(parameter) {
                if let Some(address) = captures.get(1) {
                    let address = address.as_str();
                    let address = match u16::from_str_radix(&address, 16) {
                        Ok(address) => address,
                        Err(_) => return Err(AssemblerError::InvalidAddress(line_number)),
                    };
                    let [address_low, address_high] = address.to_be_bytes();
                    match lookup_instruction(instruction, AddressingMode::Absolute) {
                        Some(byte) => {
                            program.push(vec![byte, address_low, address_high]);
                            continue;
                        }
                        None => return Err(AssemblerError::InvalidInstruction(line_number)),
                    }
                }
            } else if let Some(captures) = absolute_x_re.captures(parameter) {
                if let Some(address) = captures.get(1) {
                    let address = address.as_str();
                    let address = match u16::from_str_radix(&address, 16) {
                        Ok(address) => address,
                        Err(_) => return Err(AssemblerError::InvalidAddress(line_number)),
                    };
                    let [address_low, address_high] = address.to_be_bytes();
                    match lookup_instruction(instruction, AddressingMode::AbsoluteX) {
                        Some(byte) => {
                            program.push(vec![byte, address_low, address_high]);
                            continue;
                        }
                        None => return Err(AssemblerError::InvalidInstruction(line_number)),
                    }
                }
            } else if let Some(captures) = absolute_y_re.captures(parameter) {
                if let Some(address) = captures.get(1) {
                    let address = address.as_str();
                    let address = match u16::from_str_radix(&address, 16) {
                        Ok(address) => address,
                        Err(_) => return Err(AssemblerError::InvalidAddress(line_number)),
                    };
                    let [address_low, address_high] = address.to_be_bytes();
                    match lookup_instruction(instruction, AddressingMode::AbsoluteY) {
                        Some(byte) => {
                            program.push(vec![byte, address_low, address_high]);
                            continue;
                        }
                        None => return Err(AssemblerError::InvalidInstruction(line_number)),
                    }
                }
            } else if let Some(captures) = indirect_re.captures(parameter) {
                if let Some(address) = captures.get(1) {
                    let address = address.as_str();
                    let address = match u8::from_str_radix(&address, 16) {
                        Ok(address) => address,
                        Err(_) => return Err(AssemblerError::InvalidAddress(line_number)),
                    };
                    match lookup_instruction(instruction, AddressingMode::Indirect) {
                        Some(byte) => {
                            program.push(vec![byte, address]);
                            continue;
                        }
                        None => return Err(AssemblerError::InvalidInstruction(line_number)),
                    }
                }
            } else if let Some(captures) = indirect_x_re.captures(parameter) {
                if let Some(address) = captures.get(1) {
                    let address = address.as_str();
                    let address = match u8::from_str_radix(&address, 16) {
                        Ok(address) => address,
                        Err(_) => return Err(AssemblerError::InvalidAddress(line_number)),
                    };
                    match lookup_instruction(instruction, AddressingMode::IndirectX) {
                        Some(byte) => {
                            program.push(vec![byte, address]);
                            continue;
                        }
                        None => return Err(AssemblerError::InvalidInstruction(line_number)),
                    }
                }
            } else if let Some(captures) = indirect_y_re.captures(parameter) {
                if let Some(address) = captures.get(1) {
                    let address = address.as_str();
                    let address = match u8::from_str_radix(&address, 16) {
                        Ok(address) => address,
                        Err(_) => return Err(AssemblerError::InvalidAddress(line_number)),
                    };
                    match lookup_instruction(instruction, AddressingMode::IndirectY) {
                        Some(byte) => {
                            program.push(vec![byte, address]);
                            continue;
                        }
                        None => return Err(AssemblerError::InvalidInstruction(line_number)),
                    }
                }
            } else {
                return Err(AssemblerError::InvalidAddressingMode(line_number));
            }
        }
    }

    Ok(program)
}

#[allow(dead_code)]
pub fn run_program(program: &str) -> Result<Rc<RefCell<Bus>>, AssemblerError> {
    let program = match assemble_program(&program) {
        Ok(program) => program,
        Err(error) => return Err(error),
    };
    let mut mem: Vec<u8> = Vec::new();
    for instruction in program.iter().cloned() {
        mem.extend_from_slice(&instruction);
    }

    let mut bus = Bus::new();
    let mut location: u16 = 0;

    for byte in mem {
        bus.cpu_write(location, byte);
        location += 1;
    }

    let bus = Rc::new(RefCell::new(bus));
    let mut cpu = Mos6502::new(&bus);
    for _ in 0..program.len() {
        while !cpu.clock() {}
    }
    Ok(Rc::clone(&bus))
}

fn lookup_instruction(instruction: &str, addressing_mode: AddressingMode) -> Option<u8> {
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
            AddressingMode::Absolute => Some(0x4C),
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
