use crate::cartridge::Cartridge;
use crate::cpu_bus::Bus;
use std::fmt::{Display, Error, Formatter};
use std::ops::Deref;

const NEGATIVE_ONE: u8 = !1 + 1;

struct DataBus {
    data: u8,
}

impl DataBus {
    pub fn new() -> Self {
        DataBus { data: 0 }
    }

    pub fn read(&self) -> u8 {
        self.data
    }

    pub fn write(&mut self, data: u8) {
        self.data = data
    }
}

struct AddressBus {
    address_high: u8,
    address_low: u8,
}

impl AddressBus {
    pub fn new() -> Self {
        AddressBus {
            address_high: 0,
            address_low: 0,
        }
    }

    fn address(&self) -> u16 {
        ((self.address_high as u16) << 8) | self.address_low as u16
    }

    pub fn write(&mut self, address_high: u8, address_low: u8) {
        self.address_high = address_high;
        self.address_low = address_low;
    }
}

struct ProgramCounter {
    pch: u8,
    pcl: u8,
}

impl ProgramCounter {
    pub fn new() -> Self {
        ProgramCounter { pch: 0, pcl: 0 }
    }

    pub fn write_to_address_bus(&self, address_bus: &mut AddressBus) {
        address_bus.write(self.pch, self.pcl);
    }

    pub fn read_high_from_data_bus(&mut self, data_bus: &DataBus) {
        self.pch = data_bus.read();
    }

    pub fn read_low_from_data_bus(&mut self, data_bus: &DataBus) {
        self.pcl = data_bus.read();
    }

    pub fn read_low(&self) -> u8 {
        self.pcl
    }

    pub fn read_high(&self) -> u8 {
        self.pch
    }

    fn write(&mut self, value: u16) {
        self.pch = ((value & 0xFF00) >> 8) as u8;
        self.pcl = (value & 0x00FF) as u8;
    }

    fn wide(&self) -> u16 {
        (self.pch as u16) << 8 | self.pcl as u16
    }

    pub fn increment(&mut self) {
        self.write(self.wide().wrapping_add(1));
    }
}

impl Display for ProgramCounter {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        write!(formatter, "{:04X}", self.wide())
    }
}

struct StatusRegister {
    pub carry: bool,
    pub zero: bool,
    pub irq_disable: bool,
    pub decimal_mode: bool,
    pub brk_command: bool,
    always_one: bool,
    pub overflow: bool,
    pub negative: bool,
}

impl StatusRegister {
    pub fn new() -> Self {
        let mut p = StatusRegister {
            carry: false,
            zero: false,
            irq_disable: false,
            decimal_mode: false,
            brk_command: false,
            always_one: true,
            overflow: false,
            negative: false,
        };
        p.set(0x34);
        p
    }

    pub fn set(&mut self, byte: u8) {
        self.carry = (byte & 1) != 0;
        self.zero = (byte & (1 << 1)) != 0;
        self.irq_disable = (byte & (1 << 2)) != 0;
        self.decimal_mode = (byte & (1 << 3)) != 0;
        self.brk_command = (byte & (1 << 4)) != 0;
        // always_one cannot be changed
        self.overflow = (byte & (1 << 6)) != 0;
        self.negative = (byte & (1 << 7)) != 0;
    }

    pub fn get(&self) -> u8 {
        ((self.negative as u8) << 7)
            | ((self.overflow as u8) << 6)
            | ((self.always_one as u8) << 5)
            | ((self.brk_command as u8) << 4)
            | ((self.decimal_mode as u8) << 3)
            | ((self.irq_disable as u8) << 2)
            | ((self.zero as u8) << 1)
            | (self.carry as u8)
    }
}

impl Display for StatusRegister {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        let negative = if self.negative { "N" } else { "n" };
        let overflow = if self.overflow { "V" } else { "v" };
        let brk_command = if self.brk_command { "B" } else { "b" };
        let decimal_mode = if self.decimal_mode { "D" } else { "d" };
        let irq_disable = if self.irq_disable { "I" } else { "i" };
        let zero = if self.zero { "Z" } else { "z" };
        let carry = if self.carry { "C" } else { "c" };

        write!(
            formatter,
            "{}{}U{}{}{}{}{}",
            negative, overflow, brk_command, decimal_mode, irq_disable, zero, carry
        )
    }
}

struct InstructionRegister {
    data: u8,
}

impl InstructionRegister {
    pub fn new() -> Self {
        InstructionRegister { data: 0 }
    }

    pub fn read_from_bus(&mut self, data_bus: &DataBus) {
        self.data = data_bus.read();
    }

    pub fn decode_instruction(&self) -> Instruction {
        let low_nibble = self.data & 0x0F;
        let high_nibble = (self.data & 0xF0) >> 4;

        match low_nibble {
            0x0 => match high_nibble {
                // BRK is a 2 byte instruction, despite 6502 documentation.
                // That is, the next instruction is at PC + 2
                0x0 => Instruction::BRK(AddressingMode::Implied, 2, 7),
                0x1 => Instruction::BPL(AddressingMode::Relative, 2, 2),
                0x2 => Instruction::JSR(AddressingMode::Absolute, 3, 6),
                0x3 => Instruction::BMI(AddressingMode::Relative, 2, 2),
                0x4 => Instruction::RTI(AddressingMode::Implied, 1, 6),
                0x5 => Instruction::BVC(AddressingMode::Relative, 2, 2),
                0x6 => Instruction::RTS(AddressingMode::Implied, 1, 6),
                0x7 => Instruction::BVS(AddressingMode::Relative, 2, 2),
                0x8 => Instruction::KIL,
                0x9 => Instruction::BCC(AddressingMode::Relative, 2, 2),
                0xA => Instruction::LDY(AddressingMode::Immediate, 2, 2),
                0xB => Instruction::BCS(AddressingMode::Relative, 2, 2),
                0xC => Instruction::CPY(AddressingMode::Immediate, 2, 2),
                0xD => Instruction::BNE(AddressingMode::Relative, 2, 2),
                0xE => Instruction::CPX(AddressingMode::Immediate, 2, 2),
                0xF => Instruction::BEQ(AddressingMode::Relative, 2, 2),
                _ => unreachable!(),
            },
            0x1 => match high_nibble {
                0x0 => Instruction::ORA(AddressingMode::IndirectX, 2, 6),
                0x1 => Instruction::ORA(AddressingMode::IndirectY, 2, 5),
                0x2 => Instruction::AND(AddressingMode::IndirectX, 2, 6),
                0x3 => Instruction::AND(AddressingMode::IndirectY, 2, 5),
                0x4 => Instruction::EOR(AddressingMode::IndirectX, 2, 6),
                0x5 => Instruction::EOR(AddressingMode::IndirectY, 2, 5),
                0x6 => Instruction::ADC(AddressingMode::IndirectX, 2, 6),
                0x7 => Instruction::ADC(AddressingMode::IndirectY, 2, 5),
                0x8 => Instruction::STA(AddressingMode::IndirectX, 2, 6),
                0x9 => Instruction::STA(AddressingMode::IndirectY, 2, 6),
                0xA => Instruction::LDA(AddressingMode::IndirectX, 2, 6),
                0xB => Instruction::LDA(AddressingMode::IndirectY, 2, 5),
                0xC => Instruction::CMP(AddressingMode::IndirectX, 2, 6),
                0xD => Instruction::CMP(AddressingMode::IndirectY, 2, 5),
                0xE => Instruction::SBC(AddressingMode::IndirectX, 2, 6),
                0xF => Instruction::SBC(AddressingMode::IndirectY, 2, 5),
                _ => unreachable!(),
            },
            0x2 => match high_nibble {
                0xA => Instruction::LDX(AddressingMode::Immediate, 2, 2),
                0x0..=0x9 => Instruction::KIL,
                _ => unreachable!(),
            },
            0x3 | 0x7 | 0xB | 0xF => Instruction::KIL,
            0x4 => match high_nibble {
                0x2 => Instruction::BIT(AddressingMode::ZeroPage, 2, 3),
                0x8 => Instruction::STY(AddressingMode::ZeroPage, 2, 3),
                0x9 => Instruction::STY(AddressingMode::ZeroPageX, 2, 4),
                0xA => Instruction::LDY(AddressingMode::ZeroPage, 2, 3),
                0xB => Instruction::LDY(AddressingMode::ZeroPageX, 2, 4),
                0xC => Instruction::CPY(AddressingMode::ZeroPage, 2, 3),
                0xE => Instruction::CPX(AddressingMode::ZeroPage, 2, 3),
                0x0 | 0x1 | 0x3..=0x7 | 0xD | 0xF => Instruction::KIL,
                _ => unreachable!(),
            },
            0x5 => match high_nibble {
                0x0 => Instruction::ORA(AddressingMode::ZeroPage, 2, 3),
                0x1 => Instruction::ORA(AddressingMode::ZeroPageX, 2, 4),
                0x2 => Instruction::AND(AddressingMode::ZeroPage, 2, 3),
                0x3 => Instruction::AND(AddressingMode::ZeroPageX, 2, 4),
                0x4 => Instruction::EOR(AddressingMode::ZeroPage, 2, 3),
                0x5 => Instruction::EOR(AddressingMode::ZeroPageX, 2, 4),
                0x6 => Instruction::ADC(AddressingMode::ZeroPage, 2, 3),
                0x7 => Instruction::ADC(AddressingMode::ZeroPageX, 2, 4),
                0x8 => Instruction::STA(AddressingMode::ZeroPage, 2, 3),
                0x9 => Instruction::STA(AddressingMode::ZeroPageX, 2, 4),
                0xA => Instruction::LDA(AddressingMode::ZeroPage, 2, 3),
                0xB => Instruction::LDA(AddressingMode::ZeroPageX, 2, 4),
                0xC => Instruction::CMP(AddressingMode::ZeroPage, 2, 3),
                0xD => Instruction::CMP(AddressingMode::ZeroPageX, 2, 4),
                0xE => Instruction::SBC(AddressingMode::ZeroPage, 2, 3),
                0xF => Instruction::SBC(AddressingMode::ZeroPageX, 2, 4),
                _ => unreachable!(),
            },
            0x6 => match high_nibble {
                0x0 => Instruction::ASL(AddressingMode::ZeroPage, 2, 5),
                0x1 => Instruction::ASL(AddressingMode::ZeroPageX, 2, 6),
                0x2 => Instruction::ROL(AddressingMode::ZeroPage, 2, 5),
                0x3 => Instruction::ROL(AddressingMode::ZeroPageX, 2, 6),
                0x4 => Instruction::LSR(AddressingMode::ZeroPage, 2, 5),
                0x5 => Instruction::LSR(AddressingMode::ZeroPageX, 2, 6),
                0x6 => Instruction::ROR(AddressingMode::ZeroPage, 2, 5),
                0x7 => Instruction::ROR(AddressingMode::ZeroPageX, 2, 6),
                0x8 => Instruction::STX(AddressingMode::ZeroPage, 2, 3),
                0x9 => Instruction::STX(AddressingMode::ZeroPageY, 2, 4),
                0xA => Instruction::LDX(AddressingMode::ZeroPage, 2, 3),
                0xB => Instruction::LDX(AddressingMode::ZeroPageY, 2, 4),
                0xC => Instruction::DEC(AddressingMode::ZeroPage, 2, 5),
                0xD => Instruction::DEC(AddressingMode::ZeroPageX, 2, 6),
                0xE => Instruction::INC(AddressingMode::ZeroPage, 2, 5),
                0xF => Instruction::INC(AddressingMode::ZeroPageX, 2, 6),
                _ => unreachable!(),
            },
            0x8 => match high_nibble {
                0x0 => Instruction::PHP(AddressingMode::Implied, 1, 3),
                0x1 => Instruction::CLC(AddressingMode::Implied, 1, 2),
                0x2 => Instruction::PLP(AddressingMode::Implied, 1, 4),
                0x3 => Instruction::SEC(AddressingMode::Implied, 1, 2),
                0x4 => Instruction::PHA(AddressingMode::Implied, 1, 3),
                0x5 => Instruction::CLI(AddressingMode::Implied, 1, 2),
                0x6 => Instruction::PLA(AddressingMode::Implied, 1, 4),
                0x7 => Instruction::SEI(AddressingMode::Implied, 1, 2),
                0x8 => Instruction::DEY(AddressingMode::Implied, 1, 2),
                0x9 => Instruction::TYA(AddressingMode::Implied, 1, 2),
                0xA => Instruction::TAY(AddressingMode::Implied, 1, 2),
                0xB => Instruction::CLV(AddressingMode::Implied, 1, 2),
                0xC => Instruction::INY(AddressingMode::Implied, 1, 2),
                0xD => Instruction::CLD(AddressingMode::Implied, 1, 2),
                0xE => Instruction::INX(AddressingMode::Implied, 1, 2),
                0xF => Instruction::SED(AddressingMode::Implied, 1, 2),
                _ => unreachable!(),
            },
            0x9 => match high_nibble {
                0x0 => Instruction::ORA(AddressingMode::Immediate, 2, 2),
                0x1 => Instruction::ORA(AddressingMode::AbsoluteY, 3, 4),
                0x2 => Instruction::AND(AddressingMode::Immediate, 2, 2),
                0x3 => Instruction::AND(AddressingMode::AbsoluteY, 3, 4),
                0x4 => Instruction::EOR(AddressingMode::Immediate, 2, 2),
                0x5 => Instruction::EOR(AddressingMode::AbsoluteY, 3, 4),
                0x6 => Instruction::ADC(AddressingMode::Immediate, 2, 2),
                0x7 => Instruction::ADC(AddressingMode::AbsoluteY, 3, 4),
                0x8 => Instruction::KIL,
                0x9 => Instruction::STA(AddressingMode::AbsoluteY, 3, 5),
                0xA => Instruction::LDA(AddressingMode::Immediate, 2, 2),
                0xB => Instruction::LDA(AddressingMode::AbsoluteY, 3, 4),
                0xC => Instruction::CMP(AddressingMode::Immediate, 2, 2),
                0xD => Instruction::CMP(AddressingMode::AbsoluteY, 3, 4),
                0xE => Instruction::SBC(AddressingMode::Immediate, 2, 2),
                0xF => Instruction::SBC(AddressingMode::AbsoluteY, 3, 4),
                _ => unreachable!(),
            },
            0xA => match high_nibble {
                0x0 => Instruction::ASL(AddressingMode::Accumulator, 1, 2),
                0x2 => Instruction::ROL(AddressingMode::Accumulator, 1, 2),
                0x4 => Instruction::LSR(AddressingMode::Accumulator, 1, 2),
                0x6 => Instruction::ROR(AddressingMode::Accumulator, 1, 2),
                0x8 => Instruction::TXA(AddressingMode::Implied, 1, 2),
                0x9 => Instruction::TXS(AddressingMode::Implied, 1, 2),
                0xA => Instruction::TAX(AddressingMode::Implied, 1, 2),
                0xB => Instruction::TSX(AddressingMode::Implied, 1, 2),
                0xC => Instruction::DEX(AddressingMode::Implied, 1, 2),
                0xE => Instruction::NOP(AddressingMode::Implied, 1, 2),
                0x1 | 0x3 | 0x5 | 0x7 | 0xD | 0xF => Instruction::KIL,
                _ => unreachable!(),
            },
            0xC => match high_nibble {
                0x2 => Instruction::BIT(AddressingMode::Absolute, 3, 4),
                0x4 => Instruction::JMP(AddressingMode::Absolute, 3, 3),
                0x6 => Instruction::JMP(AddressingMode::Indirect, 3, 5),
                0x8 => Instruction::STY(AddressingMode::Absolute, 3, 4),
                0xA => Instruction::LDY(AddressingMode::Absolute, 3, 4),
                0xB => Instruction::LDY(AddressingMode::AbsoluteX, 3, 4),
                0xC => Instruction::CPY(AddressingMode::Absolute, 3, 4),
                0xE => Instruction::CPX(AddressingMode::Absolute, 3, 4),
                0x0 | 0x1 | 0x3 | 0x5 | 0x7 | 0x9 | 0xD | 0xF => Instruction::KIL,
                _ => unreachable!(),
            },
            0xD => match high_nibble {
                0x0 => Instruction::ORA(AddressingMode::Absolute, 3, 4),
                0x1 => Instruction::ORA(AddressingMode::AbsoluteX, 3, 4),
                0x2 => Instruction::AND(AddressingMode::Absolute, 3, 4),
                0x3 => Instruction::AND(AddressingMode::AbsoluteX, 3, 4),
                0x4 => Instruction::EOR(AddressingMode::Absolute, 3, 4),
                0x5 => Instruction::EOR(AddressingMode::AbsoluteX, 3, 4),
                0x6 => Instruction::ADC(AddressingMode::Absolute, 3, 4),
                0x7 => Instruction::ADC(AddressingMode::AbsoluteX, 3, 4),
                0x8 => Instruction::STA(AddressingMode::Absolute, 3, 4),
                0x9 => Instruction::STA(AddressingMode::AbsoluteX, 3, 5),
                0xA => Instruction::LDA(AddressingMode::Absolute, 3, 4),
                0xB => Instruction::LDA(AddressingMode::AbsoluteX, 3, 4),
                0xC => Instruction::CMP(AddressingMode::Absolute, 3, 4),
                0xD => Instruction::CMP(AddressingMode::AbsoluteX, 3, 4),
                0xE => Instruction::SBC(AddressingMode::Absolute, 3, 4),
                0xF => Instruction::SBC(AddressingMode::AbsoluteX, 3, 4),
                _ => unreachable!(),
            },
            0xE => match high_nibble {
                0x0 => Instruction::ASL(AddressingMode::Absolute, 3, 6),
                0x1 => Instruction::ASL(AddressingMode::AbsoluteX, 3, 7),
                0x2 => Instruction::ROL(AddressingMode::Absolute, 3, 6),
                0x3 => Instruction::ROL(AddressingMode::AbsoluteX, 3, 7),
                0x4 => Instruction::LSR(AddressingMode::Absolute, 3, 6),
                0x5 => Instruction::LSR(AddressingMode::AbsoluteX, 3, 7),
                0x6 => Instruction::ROR(AddressingMode::Absolute, 3, 6),
                0x7 => Instruction::ROR(AddressingMode::AbsoluteX, 3, 7),
                0x8 => Instruction::STX(AddressingMode::Absolute, 3, 4),
                0x9 => Instruction::KIL,
                0xA => Instruction::LDX(AddressingMode::Absolute, 3, 4),
                0xB => Instruction::LDX(AddressingMode::AbsoluteY, 3, 4),
                0xC => Instruction::DEC(AddressingMode::Absolute, 3, 6),
                0xD => Instruction::DEC(AddressingMode::AbsoluteX, 3, 7),
                0xE => Instruction::INC(AddressingMode::Absolute, 3, 6),
                0xF => Instruction::INC(AddressingMode::AbsoluteX, 3, 7),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

impl Display for InstructionRegister {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        write!(formatter, "{:02X}", self.data)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AddressingMode {
    Accumulator,
    Immediate,
    Absolute,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    AbsoluteX,
    AbsoluteY,
    Implied,
    Relative,
    Indirect,
    IndirectX,
    IndirectY,
}

/// Tuple is (addressing mode, instruction bytes, clock cycles)
#[derive(Debug)]
pub enum Instruction {
    /// Add Memory to Accumulator with Carry
    ADC(AddressingMode, u32, u32),
    /// "AND" Memory with Accumulator
    AND(AddressingMode, u32, u32),
    /// Shift Left One Bit (Memory or Accumulator)
    ASL(AddressingMode, u32, u32),

    /// Branch on Carry Clear
    BCC(AddressingMode, u32, u32),
    /// Branch on Carry Set
    BCS(AddressingMode, u32, u32),
    /// Branch on Result Zero
    BEQ(AddressingMode, u32, u32),
    /// Test Bits in Memory with Accumulator
    BIT(AddressingMode, u32, u32),
    /// Branch on Result Minus
    BMI(AddressingMode, u32, u32),
    /// Branch on Result not Zero
    BNE(AddressingMode, u32, u32),
    /// Branch on Result Plus
    BPL(AddressingMode, u32, u32),
    /// Force Break
    BRK(AddressingMode, u32, u32),
    /// Branch on Overflow Clear
    BVC(AddressingMode, u32, u32),
    /// Branch on Overflow Set
    BVS(AddressingMode, u32, u32),

    /// Clear Carry Flag
    CLC(AddressingMode, u32, u32),
    /// Clear Decimal Mode
    CLD(AddressingMode, u32, u32),
    /// Clear Interrupt Disable Bit
    CLI(AddressingMode, u32, u32),
    /// Clear Overflow Flag
    CLV(AddressingMode, u32, u32),
    /// Compare Memory and Accumulator
    CMP(AddressingMode, u32, u32),
    /// Compare Memory and Index X
    CPX(AddressingMode, u32, u32),
    /// Compare Memory and Index Y
    CPY(AddressingMode, u32, u32),

    /// Decrement Memory by One
    DEC(AddressingMode, u32, u32),
    /// Decrement Index X by One
    DEX(AddressingMode, u32, u32),
    /// Decrement Index Y by One
    DEY(AddressingMode, u32, u32),

    /// "Exclusive-OR" Memory with Accumulator
    EOR(AddressingMode, u32, u32),

    /// Increment Memory by One
    INC(AddressingMode, u32, u32),
    /// Increment Index X by One
    INX(AddressingMode, u32, u32),
    /// Increment Index Y by One
    INY(AddressingMode, u32, u32),

    /// Jump to New Location
    JMP(AddressingMode, u32, u32),
    /// Jump to New Location Saving Return Address
    JSR(AddressingMode, u32, u32),

    /// Load Accumulator with Memory
    LDA(AddressingMode, u32, u32),
    /// Load Index X with Memory
    LDX(AddressingMode, u32, u32),
    /// Load Index Y with Memory
    LDY(AddressingMode, u32, u32),
    /// Shift One Bit Right (Memory or Accumulator)
    LSR(AddressingMode, u32, u32),

    /// No Operation
    NOP(AddressingMode, u32, u32),

    /// "OR" Memory with Accumulator
    ORA(AddressingMode, u32, u32),

    /// Push Accumulator on Stack
    PHA(AddressingMode, u32, u32),
    /// Push Processor Status on Stack
    PHP(AddressingMode, u32, u32),
    /// Pull Accumulator from Stack
    PLA(AddressingMode, u32, u32),
    /// Pull Processor Status from Stack
    PLP(AddressingMode, u32, u32),

    /// Rotate One Bit Left (Memory or Accumulator)
    ROL(AddressingMode, u32, u32),
    /// Rotate One Bit Right (Memory or Accumulator)
    ROR(AddressingMode, u32, u32),
    /// Return from Interrupt
    RTI(AddressingMode, u32, u32),
    /// Return from Subroutine
    RTS(AddressingMode, u32, u32),

    /// Subtract Memory from Accumulator with Borrow
    SBC(AddressingMode, u32, u32),
    /// Set Carry Flag
    SEC(AddressingMode, u32, u32),
    /// Set Decimal Mode
    SED(AddressingMode, u32, u32),
    /// Set Interrupt Disable Status
    SEI(AddressingMode, u32, u32),
    /// Store Accumulator in Memoryj
    STA(AddressingMode, u32, u32),
    /// Store Index X in Memory
    STX(AddressingMode, u32, u32),
    /// Store Index Y in Memory
    STY(AddressingMode, u32, u32),

    /// Transfer Accumulator to Index X
    TAX(AddressingMode, u32, u32),
    /// Transfer Accumulator to Index Y
    TAY(AddressingMode, u32, u32),
    /// Transfer Stack Pointer to Index X
    TSX(AddressingMode, u32, u32),
    /// Transfer Index X to Accumulator
    TXA(AddressingMode, u32, u32),
    /// Transfer Index X to Stack Register
    TXS(AddressingMode, u32, u32),
    /// Transfer Index Y to Accumulator
    TYA(AddressingMode, u32, u32),

    /// Illegal opcode
    KIL,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut debug = format!("{:?}", self);
        debug.replace_range(3.., "");
        write!(f, "{}", debug)
    }
}

struct Accumulator {
    data: u8,
}

impl Accumulator {
    pub fn new() -> Self {
        Accumulator { data: 0 }
    }

    pub fn write_to_bus(&self, data_bus: &mut DataBus) {
        data_bus.write(self.data);
    }

    pub fn read_from_bus(&mut self, data_bus: &DataBus) {
        self.data = data_bus.read();
    }

    pub fn write(&mut self, data: u8) {
        self.data = data;
    }

    pub fn read(&self) -> u8 {
        self.data
    }
}

impl Deref for Accumulator {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

struct Alu;

impl Alu {
    pub fn new() -> Self {
        Alu
    }

    pub fn add_with_carry(
        &mut self,
        accumulator: &mut Accumulator,
        data_bus: &mut DataBus,
        p: &mut StatusRegister,
    ) {
        let accumulator_data = accumulator.read();
        let bus_data = data_bus.read();

        let sum;

        let bin = (accumulator_data as u16)
            .wrapping_add(bus_data as u16)
            .wrapping_add(p.carry as u16);

        p.carry = bin & 0x100 == 0x100;

        sum = (bin & 0xFF) as u8;
        p.zero = sum == 0;

        accumulator.write(sum);
        p.negative = sum & 0x80 == 0x80;
        p.overflow = ((accumulator_data ^ sum) & (bus_data ^ sum) & 0x80) == 0x80
    }

    pub fn subtract_with_borrow(
        &mut self,
        accumulator: &mut Accumulator,
        data_bus: &mut DataBus,
        p: &mut StatusRegister,
    ) {
        let accumulator_data = accumulator.read();
        let bus_data = data_bus.read();

        let bin = (accumulator_data as u16)
            .wrapping_add((!bus_data) as u16)
            .wrapping_add(p.carry as u16);

        // Carry = inverse of borrow
        p.carry = bin & 0x100 == 0x100;

        let sum = (bin & 0xFF) as u8;

        accumulator.write(sum);
        p.zero = sum == 0;
        p.negative = sum & 0x80 == 0x80;
        p.overflow = ((accumulator_data ^ sum) & (!bus_data ^ sum) & 0x80) == 0x80
    }
}

pub struct Mos6502 {
    /// Accumulator
    a: Accumulator,
    /// ALU
    alu: Alu,
    /// X index register
    x: u8,
    /// Y index register
    y: u8,
    /// Program counter
    pc: ProgramCounter,
    /// Stack register
    s: u8,
    /// Status register
    p: StatusRegister,
    /// Instruction register
    instruction_register: InstructionRegister,
    /// Internal data bus buffer
    data_bus: DataBus,
    /// Internal address bus
    address_bus: AddressBus,
    /// Number of cycles remaining in current instruction
    cycles: u32,
    not_irq: bool,
    not_nmi: bool,
    #[allow(dead_code)]
    not_set_overflow: bool,
    not_reset: bool,
    bus: Box<Bus>,
}

enum IndexRegister {
    X,
    Y,
}

impl Mos6502 {
    /// Initializes a new `Mos6502` processor emulator.
    pub fn new() -> Self {
        Mos6502 {
            a: Accumulator::new(),
            alu: Alu::new(),
            instruction_register: InstructionRegister::new(),
            x: 0,
            y: 0,
            pc: ProgramCounter::new(),
            s: 0xFF,
            p: StatusRegister::new(),
            data_bus: DataBus::new(),
            address_bus: AddressBus::new(),
            cycles: 0,
            not_irq: true,
            not_nmi: true,
            not_reset: true,
            not_set_overflow: true,
            bus: Box::new(Bus::new()),
        }
    }

    pub fn get_bus(&self) -> &Bus {
        &self.bus
    }

    pub fn get_bus_mut(&mut self) -> &mut Bus {
        &mut self.bus
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        self.bus.cpu_read(address)
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        self.bus.cpu_write(address, data)
    }

    pub fn reset(&mut self) {
        self.not_reset = false;
    }

    pub fn nmi(&mut self) {
        self.not_nmi = false;
    }

    #[cfg(test)]
    pub fn irq(&mut self) {
        self.not_irq = false;
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.bus.load_cartridge(cartridge);
    }

    pub fn ppu_clock(&mut self, nmi_enable: &mut bool) -> bool {
        self.bus.ppu_clock(nmi_enable)
    }

    fn write_address(&mut self, address_high: u8, address_low: u8) {
        self.address_bus.write(address_high, address_low);
    }

    fn read(&mut self) -> u8 {
        let address = self.address_bus.address();
        let data = self.bus.cpu_read(address);
        self.data_bus.write(data);
        self.data_bus.read()
    }

    fn write(&mut self) {
        self.bus
            .cpu_write(self.address_bus.address(), self.data_bus.read());
    }

    /// Runs the processor for a single clock cycle.
    ///
    /// Really, it does everything in one go on the
    /// first clock cycle and then spends the rest of
    /// the time doing nothing.
    ///
    /// Returns true if the instruction is complete.
    pub fn clock(&mut self) -> bool {
        if self.cycles == 0 {
            if !self.not_nmi {
                self.interrupt(7, 0, 0xFFFB, false, false);
                // Assume NMI should end after reset is complete
                self.not_nmi = true;
            } else if !self.not_reset {
                self.interrupt(6, 0, 0xFFFD, true, false);
                // Set stack pointer to 0xFD to mimic reset
                self.s = 0xFD;
                // Assume that reset should end after reset is complete
                self.not_reset = true;
            } else if !self.not_irq && !self.p.irq_disable {
                self.interrupt(7, 0, 0xFFFF, false, false);
                // Assume that IRQ should end after interrupt is complete
                self.not_irq = true;
            } else {
                // No interrupt, execute instruction like normal.
                self.read_instruction();
                self.execute_instruction();
            }
        }

        self.cycles -= 1;
        self.cycles == 0
    }

    fn fetch_next_byte(&mut self) -> u8 {
        self.pc.increment();
        self.pc.write_to_address_bus(&mut self.address_bus);
        self.read()
    }

    fn absolute_indexed_addressing(&mut self, index: IndexRegister) {
        let address_low = self.fetch_next_byte();
        let address_high = self.fetch_next_byte();

        let register: u8 = match index {
            IndexRegister::X => self.x,
            IndexRegister::Y => self.y,
        };
        let (address_low, carry) = address_low.overflowing_add(register);
        let address_high = if carry {
            // a carry occurred (page boundary crossed), need to add one
            // to high byte of address and use additional cycle
            self.cycles += 1;
            address_high.wrapping_add(1)
        } else {
            address_high
        };

        self.address_bus.write(address_high, address_low);
    }

    fn do_addressing_mode(&mut self, mode: AddressingMode) {
        self.do_addressing_mode_with_branch(mode, false);
    }

    fn do_addressing_mode_with_branch(&mut self, mode: AddressingMode, take_branch: bool) {
        match mode {
            AddressingMode::Absolute => {
                let address_low = self.fetch_next_byte();
                let address_high = self.fetch_next_byte();
                self.write_address(address_high, address_low);
            }
            AddressingMode::Indirect => {
                let address_low = self.fetch_next_byte();
                let address_high = self.fetch_next_byte();

                self.write_address(address_high, address_low);
                let new_address_low = self.read();
                self.write_address(address_high, address_low.wrapping_add(1));
                let new_address_high = self.read();
                self.write_address(new_address_high, new_address_low);
            }
            AddressingMode::AbsoluteX => self.absolute_indexed_addressing(IndexRegister::X),
            AddressingMode::AbsoluteY => self.absolute_indexed_addressing(IndexRegister::Y),
            AddressingMode::Accumulator => return,
            AddressingMode::Immediate => {
                self.pc.increment();
                self.pc.write_to_address_bus(&mut self.address_bus);
            }
            AddressingMode::Implied => return,
            AddressingMode::IndirectX => {
                // Indexed indirect addressing with register X
                let zero_page_offset = self.fetch_next_byte();
                let zero_page_offset = zero_page_offset.wrapping_add(self.x);
                self.write_address(0, zero_page_offset);
                let address_low = self.read();

                let zero_page_offset = zero_page_offset.wrapping_add(1);
                self.write_address(0, zero_page_offset);
                let address_high = self.read();

                self.write_address(address_high, address_low);
            }
            AddressingMode::IndirectY => {
                // Indirect indexed addressing with register Y
                let zero_page_offset = self.fetch_next_byte();
                self.write_address(0, zero_page_offset);
                let address_low = self.read();

                let zero_page_offset = zero_page_offset.wrapping_add(1);
                self.write_address(0, zero_page_offset);
                let address_high = self.read();

                let (address_low, carry) = address_low.overflowing_add(self.y);
                let address_high = if carry {
                    // a carry occurred (page boundary crossed), need to add one
                    // to high byte of address and use additional cycle
                    self.cycles += 1;
                    address_high.wrapping_add(1)
                } else {
                    address_high
                };
                self.write_address(address_high, address_low);
            }
            AddressingMode::Relative => {
                let offset = self.fetch_next_byte();
                let offset_negative = offset & 0x80 == 0x80;

                // PCL + offset -> PCL
                let (pcl, carry) = self.pc.read_low().overflowing_add(offset);
                if take_branch {
                    self.data_bus.write(pcl);
                    self.pc.read_low_from_data_bus(&self.data_bus);
                }

                // If the offset was negative, we expect a carry
                // when no page boundary is crossed
                if offset_negative && carry {
                    return;
                }

                // If the offset was positive, we expect no carry
                // when no page boundary is crossed.
                if !offset_negative && !carry {
                    return;
                }

                // Page boundary crossed, additional cycle needed
                // and we must calculate the new PCH.

                // PCH + 0 + carry -> PCH
                let increment = if offset_negative { NEGATIVE_ONE } else { 1 };
                let pch = self.pc.read_high().wrapping_add(increment);
                if take_branch {
                    self.cycles += 1;
                    self.data_bus.write(pch);
                    self.pc.read_high_from_data_bus(&self.data_bus);
                }
            }
            AddressingMode::ZeroPage => {
                let zero_page_offset = self.fetch_next_byte();
                self.write_address(0, zero_page_offset);
            }
            AddressingMode::ZeroPageX => {
                let old_zero_page_offset = self.fetch_next_byte();
                let zero_page_offset = old_zero_page_offset.wrapping_add(self.x);
                self.write_address(0, zero_page_offset);
            }
            AddressingMode::ZeroPageY => {
                let old_zero_page_offset = self.fetch_next_byte();
                let zero_page_offset = old_zero_page_offset.wrapping_add(self.y);
                self.write_address(0, zero_page_offset);
            }
        }
    }

    fn branch(&mut self, branch: bool, mode: AddressingMode, cycles: u32) {
        self.cycles = cycles;

        if branch {
            // Branch taken, add an additional cycle.
            self.cycles += 1;
        }

        self.do_addressing_mode_with_branch(mode, branch);
    }

    fn compare(&mut self, mode: AddressingMode, operand: u8, cycles: u32) {
        self.cycles = cycles;

        self.do_addressing_mode(mode);
        self.read();
        let memory = self.data_bus.read();

        let result = operand.wrapping_sub(memory);

        self.p.zero = result == 0;
        self.p.negative = result & 0x80 == 0x80;
        self.p.carry = operand >= memory;
    }

    fn increment(&mut self, operand: u8, by: u8, cycles: u32) -> u8 {
        self.cycles = cycles;

        let result = operand.wrapping_add(by);

        self.p.zero = result == 0;
        self.p.negative = result & 0x80 == 0x80;

        result
    }

    fn jump(&mut self, mode: AddressingMode, cycles: u32) {
        self.cycles = cycles;

        self.do_addressing_mode(mode);
        let pc = self.address_bus.address();

        // Subtract one to counteract the standard PC increment.
        let pc = pc.wrapping_sub(1);
        self.pc.write(pc);
    }

    fn interrupt(
        &mut self,
        cycles: u32,
        bytes: u32,
        interrupt_vector: u16,
        suppress_writes: bool,
        brk_command: bool,
    ) {
        self.cycles = cycles;

        let address = self.pc.wide();
        // Store the next instruction in the stack
        let address = address.wrapping_add(bytes as u16);
        let high = ((address & 0xFF00) >> 8) as u8;
        let low = (address & 0x00FF) as u8;

        // A reset suppresses writes to memory.
        if !suppress_writes {
            self.write_address(0x01, self.s);
            self.s = self.s.wrapping_sub(1);
            self.data_bus.write(high);
            self.write();
        }

        // A reset suppresses writes to memory.
        if !suppress_writes {
            self.write_address(0x01, self.s);
            self.s = self.s.wrapping_sub(1);
            self.data_bus.write(low);
            self.write();
        }

        // A reset suppresses writes to memory.
        if !suppress_writes {
            self.write_address(0x01, self.s);
            self.s = self.s.wrapping_sub(1);
            let mut p = self.p.get();

            // The B flag should be set if this interrupt is from a BRK instruction,
            // otherwise it should be cleared.
            if brk_command {
                p |= 0x10;
            } else {
                p &= !0x10;
            }

            self.data_bus.write(p);
            self.write();
        }

        let vector_high = ((interrupt_vector & 0xFF00) >> 8) as u8;
        let vector_low = (interrupt_vector & 0xFF) as u8;

        self.write_address(vector_high, vector_low);
        self.read();
        self.pc.read_high_from_data_bus(&self.data_bus);

        let interrupt_vector = interrupt_vector.wrapping_sub(1);
        let vector_high = ((interrupt_vector & 0xFF00) >> 8) as u8;
        let vector_low = (interrupt_vector & 0xFF) as u8;

        self.write_address(vector_high, vector_low);
        self.read();
        self.pc.read_low_from_data_bus(&self.data_bus);

        self.p.irq_disable = true;
    }

    fn read_instruction(&mut self) {
        self.pc.write_to_address_bus(&mut self.address_bus);
        self.read();
        self.instruction_register.read_from_bus(&self.data_bus);
    }

    fn execute_instruction(&mut self) {
        let instruction = self.instruction_register.decode_instruction();

        match instruction {
            Instruction::ADC(mode, _, cycles) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.read();
                self.alu
                    .add_with_carry(&mut self.a, &mut self.data_bus, &mut self.p);
            }
            Instruction::AND(mode, _, cycles) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);

                let operand = self.read();
                let result = operand & self.a.read();

                self.a.write(result);

                self.p.zero = result == 0;
                self.p.negative = result & 0x80 == 0x80;
            }
            Instruction::ASL(mode, _, cycles) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                let operand = if mode == AddressingMode::Accumulator {
                    self.a.read()
                } else {
                    self.read()
                };
                let result = operand << 1;
                self.data_bus.write(result);

                self.p.zero = result == 0;
                self.p.negative = result & 0x80 == 0x80;
                self.p.carry = operand & 0x80 == 0x80;

                if mode == AddressingMode::Accumulator {
                    self.a.read_from_bus(&self.data_bus);
                } else {
                    self.write();
                }
            }
            Instruction::BCC(mode, _, cycles) => self.branch(!self.p.carry, mode, cycles),
            Instruction::BCS(mode, _, cycles) => self.branch(self.p.carry, mode, cycles),
            Instruction::BEQ(mode, _, cycles) => self.branch(self.p.zero, mode, cycles),
            Instruction::BIT(mode, _, cycles) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                let operand = self.read();
                self.p.negative = operand & 0x80 == 0x80;
                self.p.overflow = operand & 0x40 == 0x40;
                self.p.zero = operand & self.a.read() == 0;
            }
            Instruction::BMI(mode, _, cycles) => self.branch(self.p.negative, mode, cycles),
            Instruction::BNE(mode, _, cycles) => self.branch(!self.p.zero, mode, cycles),
            Instruction::BPL(mode, _, cycles) => self.branch(!self.p.negative, mode, cycles),
            Instruction::BRK(_, bytes, cycles) => {
                self.interrupt(cycles, bytes, 0xFFFF, false, true);
            }
            Instruction::BVC(mode, _, cycles) => self.branch(!self.p.overflow, mode, cycles),
            Instruction::BVS(mode, _, cycles) => self.branch(self.p.overflow, mode, cycles),
            Instruction::CLC(_, _, cycles) => {
                self.cycles = cycles;
                self.p.carry = false;
            }
            Instruction::CLD(_, _, cycles) => {
                self.cycles = cycles;
                self.p.decimal_mode = false;
            }
            Instruction::CLI(_, _, cycles) => {
                self.cycles = cycles;
                self.p.irq_disable = false;
            }
            Instruction::CLV(_, _, cycles) => {
                self.cycles = cycles;
                self.p.overflow = false;
            }
            Instruction::CMP(mode, _, cycles) => {
                let operand = self.a.read();
                self.compare(mode, operand, cycles)
            }
            Instruction::CPX(mode, _, cycles) => self.compare(mode, self.x, cycles),
            Instruction::CPY(mode, _, cycles) => self.compare(mode, self.y, cycles),
            Instruction::DEC(mode, _, cycles) => {
                self.do_addressing_mode(mode);
                let memory = self.read();
                let result = self.increment(memory, NEGATIVE_ONE, cycles);

                self.data_bus.write(result);
                self.write();
            }
            Instruction::DEX(_, _, cycles) => {
                self.x = self.increment(self.x, NEGATIVE_ONE, cycles);
            }
            Instruction::DEY(_, _, cycles) => {
                self.y = self.increment(self.y, NEGATIVE_ONE, cycles);
            }
            Instruction::EOR(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);
                let operand = self.read();

                let result = self.a.read() ^ operand;
                self.a.write(result);

                self.p.zero = result == 0;
                self.p.negative = result & 0x80 == 0x80;
            }
            Instruction::INC(mode, _, cycles) => {
                self.do_addressing_mode(mode);
                let operand = self.read();

                let result = self.increment(operand, 1, cycles);
                self.data_bus.write(result);
                self.write();
            }
            Instruction::INX(_, _, cycles) => {
                self.x = self.increment(self.x, 1, cycles);
            }
            Instruction::INY(_, _, cycles) => {
                self.y = self.increment(self.y, 1, cycles);
            }
            Instruction::JMP(mode, _, cycles) => self.jump(mode, cycles),
            Instruction::JSR(mode, bytes, cycles) => {
                let next_address = self
                    .pc
                    .wide()
                    .wrapping_add((bytes & 0xFFFF) as u16)
                    // We want the next address to be the last byte of this instruction,
                    // not the first byte of the next, so we subtract one.
                    .wrapping_sub(1);
                let next_address_high = ((next_address & 0xFF00) >> 8) as u8;
                let next_address_low = (next_address & 0x00FF) as u8;

                // Save next instruction location to the stack.
                self.write_address(0x01, self.s);
                self.data_bus.write(next_address_high);
                self.write();
                self.s = self.s.wrapping_sub(1);

                self.write_address(0x01, self.s);
                self.data_bus.write(next_address_low);
                self.write();
                self.s = self.s.wrapping_sub(1);

                self.jump(mode, cycles);
            }
            Instruction::LDA(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);
                self.read();
                self.a.read_from_bus(&self.data_bus);
                let a = self.a.read();
                self.p.negative = a & 0x80 == 0x80;
                self.p.zero = a == 0;
            }
            Instruction::LDX(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);
                self.x = self.read();
                self.p.negative = self.x & 0x80 == 0x80;
                self.p.zero = self.x == 0;
            }
            Instruction::LDY(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);
                self.y = self.read();
                self.p.negative = self.y & 0x80 == 0x80;
                self.p.zero = self.y == 0;
            }
            Instruction::LSR(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);
                let operand = if mode == AddressingMode::Accumulator {
                    self.a.read()
                } else {
                    self.read()
                };
                self.p.carry = operand & 0x01 == 0x01;

                let result = operand >> 1;
                self.data_bus.write(result);
                self.p.zero = result == 0x00;
                self.p.negative = false;

                if mode == AddressingMode::Accumulator {
                    self.a.read_from_bus(&self.data_bus);
                } else {
                    self.write();
                }
            }
            Instruction::NOP(_, _, cycles) => {
                self.cycles = cycles;
            }
            Instruction::ORA(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);

                let memory = self.read();
                let result = self.a.read() | memory;

                self.a.write(result);
                self.p.zero = result == 0;
                self.p.negative = result & 0x80 == 0x80;
            }
            Instruction::PHA(_, _, cycles) => {
                self.cycles = cycles;

                self.write_address(0x01, self.s);
                self.a.write_to_bus(&mut self.data_bus);
                self.write();

                self.s = self.s.wrapping_sub(1);
            }
            Instruction::PHP(_, _, cycles) => {
                self.cycles = cycles;

                self.write_address(0x01, self.s);
                // Bit 4 is always set when pushing
                let p = self.p.get() | 0x10;
                self.data_bus.write(p);
                self.write();
                self.s = self.s.wrapping_sub(1);
            }
            Instruction::PLA(_, _, cycles) => {
                self.cycles = cycles;

                self.s = self.s.wrapping_add(1);
                self.write_address(0x01, self.s);
                let value = self.read();
                self.a.write(value);
                self.p.negative = value & 0x80 == 0x80;
                self.p.zero = value == 0;
            }
            Instruction::PLP(_, _, cycles) => {
                self.cycles = cycles;

                self.s = self.s.wrapping_add(1);
                self.write_address(0x01, self.s);
                let value = self.read();
                self.p.set(value);
            }
            Instruction::ROL(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);
                let operand = if mode == AddressingMode::Accumulator {
                    self.a.read()
                } else {
                    self.read()
                };
                // Shift left and make bit 0 the carry bit
                let result = operand << 1 | (self.p.carry as u8);
                // The new carry bit is the old bit 7.
                self.p.carry = ((operand & 0x80) >> 7) != 0;
                self.p.negative = result & 0x80 == 0x80;
                self.p.zero = result == 0;

                self.data_bus.write(result);

                if mode == AddressingMode::Accumulator {
                    self.a.read_from_bus(&self.data_bus);
                } else {
                    self.write();
                }
            }
            Instruction::ROR(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);
                let operand = if mode == AddressingMode::Accumulator {
                    self.a.read()
                } else {
                    self.read()
                };
                // Shift right and make bit 7 the carry bit
                let result = operand >> 1 | ((self.p.carry as u8) << 7);
                // The new carry is the old bit 0.
                self.p.carry = operand & 0x01 != 0;
                self.p.negative = result & 0x80 == 0x80;
                self.p.zero = result == 0;

                self.data_bus.write(result);

                if mode == AddressingMode::Accumulator {
                    self.a.read_from_bus(&self.data_bus);
                } else {
                    self.write();
                }
            }
            Instruction::RTI(_, _, cycles) => {
                self.cycles = cycles;

                self.s = self.s.wrapping_add(1);
                self.write_address(0x01, self.s);
                let data = self.read();
                self.p.set(data);

                self.s = self.s.wrapping_add(1);
                self.write_address(0x01, self.s);
                self.read();
                self.pc.read_low_from_data_bus(&self.data_bus);

                self.s = self.s.wrapping_add(1);
                self.write_address(0x01, self.s);
                self.read();
                self.pc.read_high_from_data_bus(&self.data_bus);

                // Subtract one from program counter to counteract
                // standard increment
                let pc = self.pc.wide();
                self.pc.write(pc.wrapping_sub(1));
            }
            Instruction::RTS(_, _, cycles) => {
                self.cycles = cycles;

                self.s = self.s.wrapping_add(1);
                self.write_address(0x01, self.s);
                self.read();
                self.pc.read_low_from_data_bus(&self.data_bus);

                self.s = self.s.wrapping_add(1);
                self.write_address(0x01, self.s);
                self.read();
                self.pc.read_high_from_data_bus(&self.data_bus);
            }
            Instruction::SBC(mode, _, cycles) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.read();
                self.alu
                    .subtract_with_borrow(&mut self.a, &mut self.data_bus, &mut self.p);
            }
            Instruction::SEC(_, _, cycles) => {
                self.cycles = cycles;
                self.p.carry = true;
            }
            Instruction::SED(_, _, cycles) => {
                self.cycles = cycles;
                self.p.decimal_mode = true;
            }
            Instruction::SEI(_, _, cycles) => {
                self.cycles = cycles;
                self.p.irq_disable = true;
            }
            Instruction::STA(mode, _, cycles) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.a.write_to_bus(&mut self.data_bus);
                self.write();
            }
            Instruction::STX(mode, _, cycles) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.data_bus.write(self.x);
                self.write();
            }
            Instruction::STY(mode, _, cycles) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.data_bus.write(self.y);
                self.write();
            }
            Instruction::TAX(_, _, cycles) => {
                self.cycles = cycles;

                self.x = self.a.read();
                self.p.negative = self.x & 0x80 == 0x80;
                self.p.zero = self.x == 0;
            }
            Instruction::TAY(_, _, cycles) => {
                self.cycles = cycles;

                self.y = self.a.read();
                self.p.negative = self.y & 0x80 == 0x80;
                self.p.zero = self.y == 0;
            }
            Instruction::TSX(_, _, cycles) => {
                self.cycles = cycles;

                self.x = self.s;
                self.p.negative = self.x & 0x80 == 0x80;
                self.p.zero = self.x == 0;
            }
            Instruction::TXA(_, _, cycles) => {
                self.cycles = cycles;

                self.a.write(self.x);
                self.p.negative = self.a.read() & 0x80 == 0x80;
                self.p.zero = self.a.read() == 0x00;
            }
            Instruction::TXS(_, _, cycles) => {
                self.cycles = cycles;

                self.s = self.x;
            }
            Instruction::TYA(_, _, cycles) => {
                self.cycles = cycles;

                self.a.write(self.y);
                self.p.negative = self.a.read() & 0x80 == 0x80;
                self.p.zero = self.a.read() == 0x00;
            }
            Instruction::KIL => panic!(
                "{} instruction not implemented at address {:04X}",
                self.instruction_register,
                self.pc.wide()
            ),
        };

        self.pc.increment();
    }
}

#[cfg(test)]
mod tests {
    use super::Mos6502;
    use crate::assembler::{self, AssemblerError};

    fn run_program(program: &str) -> Box<Mos6502> {
        match assembler::run_program(program) {
            Ok(cpu) => cpu,
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
        assert_eq!(cpu.cpu_read(0xFF), 2, "0x1 + 0x1 = 0x2");
        assert_eq!(cpu.cpu_read(0x01FF) & 0x01, 0x00, "carry bit cleared");

        let mut cpu = run_program(
            "
            LDA #$FF
            ADC #$FF
            STA $FF
            PHP
        ",
        );
        assert_eq!(cpu.cpu_read(0xFF), 0xFE, "0xFF + 0xFF = 0xFE");
        assert_eq!(cpu.cpu_read(0x01FF) & 0x01, 0x01, "carry bit set");
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
        assert_eq!(cpu.cpu_read(0xFF), 0x00, "(0xAA & 0x55) = 0x00");
        let status = cpu.cpu_read(0x01FF);
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
        assert_eq!(cpu.cpu_read(0xFF), 0x80, "0xFF & 0x80 = 0x80");
        assert_eq!(cpu.cpu_read(0x01FF) & 0x80, 0x80, "negative bit set");
        assert_eq!(cpu.cpu_read(0x01FF) & 0x02, 0x00, "zero bit not set");
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
        let status = cpu.cpu_read(0x01FF);
        assert_eq!(cpu.cpu_read(0xFF), 0xFE, "asl result correct");
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

        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "branch not taken");

        let mut cpu = run_program(
            "
        LDA #$FE
        ADC #$01 // Result is 0xFF, carry cleared
        BCC $02  // Branch should be taken to STA $FF
        LDA #$FA
        STA $FF  // 0xFF stored to $FF
        ",
        );

        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "branch taken");
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

        assert_eq!(cpu.cpu_read(0xFF), 0x01, "branch taken");

        let mut cpu = run_program(
            "
        LDA #$FE
        ADC #$01 // Result is 0xFF, carry cleared
        BCS $02  // Branch should not be taken, next line executes
        LDA #$FA
        STA $FF  // 0xFA stored to $FF
        ",
        );

        assert_eq!(cpu.cpu_read(0xFF), 0xFA, "branch not taken");
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

        assert_eq!(cpu.cpu_read(0xFF), 0x00, "branch taken");
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

        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "branch not taken");
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

        let status = cpu.cpu_read(0x01FF);
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

        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "branch taken");

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

        assert_eq!(cpu.cpu_read(0xFF), 0x02, "branch not taken");
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

        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "branch not taken");

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

        assert_eq!(cpu.cpu_read(0xFF), 0x01, "branch taken");
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

        assert_eq!(cpu.cpu_read(0xFF), 0x01, "branch taken");

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

        assert_eq!(cpu.cpu_read(0xFF), 0x03, "branch not taken");
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
            cpu.cpu_read(0x01FF),
            0x00,
            "address after BRK stored on stack"
        );
        assert_eq!(
            cpu.cpu_read(0x01FE),
            0x07,
            "address after BRK stored on stack"
        );
        assert_eq!(
            cpu.cpu_read(0x01FD) & 0x02,
            0x02,
            "zero flag stored on stack"
        );
        assert_eq!(
            cpu.cpu_read(0x01FD) & 0x01,
            0x01,
            "carry flag stored on stack"
        );
    }

    #[test]
    fn bvc() {
        let mut cpu = run_program(
            "
            LDA #$80
            ADC #$80 // Result is 0x04, overflow set
            BVC $02  // Branch should not be taken, execute next instruction
            LDA #$FF
            STA $FF  // Store 0xFF in $FF 
        ",
        );

        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "branch not taken");

        let mut cpu = run_program(
            "
            LDA #$01
            ADC #$05 // Overflow not set
            BVC $02  // Branch should be taken, continue with STA $FF
            LDA #$FF
            STA $FF  // Store 0x06 in $FF 
        ",
        );

        assert_eq!(cpu.cpu_read(0xFF), 0x06, "branch taken");
    }

    #[test]
    fn bvs() {
        let mut cpu = run_program(
            "
            LDA #$80
            ADC #$84 // Overflow set
            BVS $02  // Branch should be taken, continue with STA $FF
            LDA #$FF
            STA $FF  // Store 0x4 in $FF 
        ",
        );

        assert_eq!(cpu.cpu_read(0xFF), 0x04, "branch taken");

        let mut cpu = run_program(
            "
            LDA #$01
            ADC #$05 // Result is 0x06, overflow not set
            BVS $02  // Branch should not be taken, continue with STA $FF
            LDA #$FF
            STA $FF  // Store 0xFF in $FF 
        ",
        );

        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "branch taken");
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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");

        let mut cpu = run_program(
            "
            LDA #$10
            CMP #$10
            PHP
            ",
        );

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x02, "zero flag set");

        let mut cpu = run_program(
            "
            LDA #$10
            CMP #$11
            PHP
            ",
        );

        let status = cpu.cpu_read(0x01FF);
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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");

        let mut cpu = run_program(
            "
            LDX #$10
            CPX #$10
            PHP
            ",
        );

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x02, "zero flag set");

        let mut cpu = run_program(
            "
            LDX #$10
            CPX #$11
            PHP
            ",
        );

        let status = cpu.cpu_read(0x01FF);
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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");

        let mut cpu = run_program(
            "
            LDY #$10
            CPY #$10
            PHP
            ",
        );

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x02, "zero flag set");

        let mut cpu = run_program(
            "
            LDY #$10
            CPY #$11
            PHP
            ",
        );

        let status = cpu.cpu_read(0x01FF);
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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(cpu.cpu_read(0xFF), 0x00, "correct result");

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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "correct result");
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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(cpu.cpu_read(0xFF), 0x00, "correct result");

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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "correct result");
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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(cpu.cpu_read(0xFF), 0x00, "correct result");

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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "correct result");
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

        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "0x55 xor 0xAA = 0xFF");
        assert_eq!(cpu.cpu_read(0x01FF) & 0x80, 0x80, "negative bit set");

        let mut cpu = run_program(
            "
            LDA #$FF
            EOR #$FF
            STA $FF
            PHP
        ",
        );

        assert_eq!(cpu.cpu_read(0xFF), 0x00, "0xFF xor 0xFF = 0x00");
        assert_eq!(cpu.cpu_read(0x01FF) & 0x02, 0x02, "zero bit set");
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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(cpu.cpu_read(0xFF), 0x00, "correct result");

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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "correct result");
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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(cpu.cpu_read(0xFF), 0x00, "correct result");

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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "correct result");
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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(cpu.cpu_read(0xFF), 0x00, "correct result");

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

        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn jmp() {
        let mut cpu = run_program(
            "
            JMP $0900
            NOP
            NOP
            LDA #$FF // These two lines should not execute
            STA $FF  // so $FF should be empty.
            NOP
        ",
        );

        assert_eq!(cpu.cpu_read(0xFF), 0x00, "load and store jumped over");
    }

    #[test]
    fn jsr() {
        let mut cpu = run_program(
            "
            JSR $0900
            LDA #$FF
            STA $FF
            LDA #$FF
            STA $FE
        ",
        );

        assert_ne!(cpu.cpu_read(0xFF), 0xFF, "first store skipped");
        assert_ne!(cpu.cpu_read(0xFE), 0xFF, "second store skipped");
        assert_eq!(cpu.cpu_read(0x01FF), 0x00, "high byte = 0x00");
        assert_eq!(cpu.cpu_read(0x01FE), 0x02, "low byte = 0x02");
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

        assert_eq!(cpu.cpu_read(0xFF), 0x7F, "0xFF >> 1 = 0x7F");
        let status = cpu.cpu_read(0x01FF);
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

        assert_eq!(cpu.cpu_read(0xFF), 0x00, "0x01 >> 1 = 0x00");
        let status = cpu.cpu_read(0x01FF);
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

        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "0xAA | 0x55 = 0xFF");
        let status = cpu.cpu_read(0x01FF);
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
        assert_eq!(cpu.cpu_read(0xFF), 0x00, "0x00 | 0x00 = 0x00");
        let status = cpu.cpu_read(0x01FF);
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

        assert_eq!(cpu.cpu_read(0x01FF), 0xFF, "accumulator pushed on stack");
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

        assert_eq!(cpu.cpu_read(0x01FF), 0xFF, "accumulator pulled from stack");
    }

    #[test]
    fn rol() {
        let mut cpu = run_program(
            "
        LDA #$FF
        STA $FF
        ROL $FF
        PHP
        ",
        );

        assert_eq!(cpu.cpu_read(0x01FF) & 0x01, 0x01, "carry bit set");
        assert_eq!(cpu.cpu_read(0xFF), 0xFE, "correct result");

        let mut cpu = run_program(
            "
        LDA #$FF
        STA $FF
        SEC
        ROL $FF
        PHP
        ",
        );

        assert_eq!(cpu.cpu_read(0x01FF) & 0x01, 0x01, "carry bit set");
        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn ror() {
        let mut cpu = run_program(
            "
            LDA #$FF
            STA $FF
            ROR $FF
            PHP
        ",
        );

        assert_eq!(cpu.cpu_read(0x01FF) & 0x01, 0x01, "carry bit set");
        assert_eq!(cpu.cpu_read(0xFF), 0x7F, "correct result");

        let mut cpu = run_program(
            "
            LDA #$FF
            STA $FF
            SEC
            ROR $FF
            PHP
        ",
        );

        assert_eq!(cpu.cpu_read(0x01FF) & 0x01, 0x01, "carry bit set");
        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn tax() {
        let mut cpu = run_program(
            "
            LDA #$FF
            TAX
            STX $FF
            PHP
        ",
        );

        assert_eq!(cpu.cpu_read(0xFF), 0xFF, "correct result");
        assert_eq!(cpu.cpu_read(0x01FF) & 0x80, 0x80, "negative bit set");
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
        assert_eq!(cpu.cpu_read(0xFF), 0x71, "0x76 - 0x05 = 0x71");
        let status = cpu.cpu_read(0x01FF);
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
        assert_eq!(cpu.cpu_read(0xFF), 0xFB, "0x5 - 0xA = -0x5 (0xFB)");
        let status = cpu.cpu_read(0x01FF);
        assert_eq!(status & 0x01, 0x00, "borrow (carry not set)");
        assert_eq!(status & 0x80, 0x80, "negative bit set");
        assert_eq!(status & 0x02, 0x00, "zero bit not set");
    }

    #[test]
    fn irq() {
        let mut cpu = Mos6502::new();

        let program = assembler::assemble_program(
            "
            CLI
            LDX #$00
            INX
            JMP $0300 // Jump back to INX, keep incrementing
            STX $FF   // Should never happen unless interrupt works
            RTI
        ",
        )
        .expect("Encountered assembler error");

        let mut mem: Vec<u8> = Vec::new();
        for instruction in program.iter().cloned() {
            mem.extend_from_slice(&instruction);
        }

        let mut location = 0;

        for byte in mem {
            cpu.get_bus_mut().cpu_write(location, byte);
            location += 1;
        }

        // Set interrupt vector to start at RTI
        cpu.get_bus_mut().cpu_write(0xFFFF, 0x00); // Address high
        cpu.get_bus_mut().cpu_write(0xFFFE, 0x06); // Address low

        // Do loop for a while
        for _ in 0..20 {
            while !cpu.clock() {}
        }

        // Interrupt
        cpu.irq();

        // Do interrupt and two instructions
        for _ in 0..3 {
            while !cpu.clock() {}
        }

        assert_ne!(cpu.cpu_read(0x00FF), 0, "data stored in 0xFF");
    }
}
