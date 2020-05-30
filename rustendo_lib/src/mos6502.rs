use std::cell::RefCell;
use std::rc::Rc;

pub struct AddressBus {
    adh: u8,
    adl: u8,
}

impl AddressBus {
    pub fn new() -> Self {
        AddressBus { adl: 0, adh: 0 }
    }

    pub fn write_wide(&mut self, address: u16) {
        let [high, low] = address.to_be_bytes();
        self.adh = high;
        self.adl = low;
    }

    pub fn write(&mut self, high: u8, low: u8) {
        self.adh = high;
        self.adl = low;
    }

    pub fn read(&self) -> (u8, u8) {
        (self.adh, self.adl)
    }

    pub fn read_wide(&self) -> u16 {
        (self.adh as u16) << 8 | self.adl as u16
    }
}

pub struct ProgramCounter {
    pch: u8,
    pcl: u8,
    data_bus: Rc<RefCell<DataBus>>,
    address_bus: Rc<RefCell<AddressBus>>,
}

impl ProgramCounter {
    pub fn new(data_bus: Rc<RefCell<DataBus>>, address_bus: Rc<RefCell<AddressBus>>) -> Self {
        ProgramCounter {
            pch: 0,
            pcl: 0,
            data_bus,
            address_bus,
        }
    }

    pub fn write_to_address_bus(&self) {
        self.address_bus.borrow_mut().write(self.pch, self.pcl);
    }

    pub fn read_high_from_bus(&mut self) {
        self.pch = self.data_bus.borrow().read();
    }

    pub fn read_low_from_bus(&mut self) {
        self.pcl = self.data_bus.borrow().read();
    }

    pub fn write(&mut self, value: u16) {
        let [high, low] = value.to_be_bytes();
        self.pch = high;
        self.pcl = low;
    }

    pub fn high(&self) -> u8 {
        self.pch
    }

    pub fn low(&self) -> u8 {
        self.pcl
    }

    pub fn wide(&self) -> u16 {
        (self.pch as u16) << 8 | self.pcl as u16
    }

    pub fn increment(&mut self) {
        self.write(self.wide().wrapping_add(1));
    }
}

pub struct StatusRegister {
    carry: bool,
    zero: bool,
    irq_disable: bool,
    decimal_mode: bool,
    brk_command: bool,
    overflow: bool,
    always_one: bool,
    negative: bool,
}

impl StatusRegister {
    pub fn new() -> Self {
        let mut p = StatusRegister {
            carry: false,
            zero: false,
            irq_disable: false,
            decimal_mode: false,
            brk_command: false,
            always_one: false,
            overflow: false,
            negative: false,
        };
        p.set(0x34);
        p
    }

    pub fn set_carry(&mut self, carry: bool) {
        self.carry = carry;
    }

    pub fn get_carry(&self) -> bool {
        self.carry
    }

    pub fn set_zero(&mut self, zero: bool) {
        self.zero = zero;
    }

    pub fn get_zero(&self) -> bool {
        self.zero
    }

    pub fn set_negative(&mut self, negative: bool) {
        self.negative = negative;
    }

    pub fn get_negative(&self) -> bool {
        self.negative
    }

    pub fn set_overflow(&mut self, overflow: bool) {
        self.overflow = overflow
    }

    pub fn get_overflow(&self) -> bool {
        self.overflow
    }

    pub fn set_decimal_mode(&mut self, decimal_mode: bool) {
        self.decimal_mode = decimal_mode
    }

    pub fn get_decimal_mode(&self) -> bool {
        self.decimal_mode
    }

    pub fn set(&mut self, byte: u8) {
        self.carry = (byte & (1 << 0)) != 0;
        self.zero = (byte & (1 << 1)) != 0;
        self.irq_disable = (byte & (1 << 2)) != 0;
        self.decimal_mode = (byte & (1 << 3)) != 0;
        self.brk_command = (byte & (1 << 4)) != 0;
        // always_one cannot be changed
        self.overflow = (byte & (1 << 6)) != 0;
        self.negative = (byte & (1 << 7)) != 0;
    }

    pub fn get(&self) -> u8 {
        (self.carry as u8)
            | ((self.zero as u8) << 1)
            | ((self.irq_disable as u8) << 2)
            | ((self.decimal_mode as u8) << 3)
            | ((self.brk_command as u8) << 4)
            | ((self.always_one as u8) << 5)
            | ((self.overflow as u8) << 6)
            | ((self.negative as u8) << 7)
    }
}

pub struct Accumulator {
    register: u8,
    data_bus: Rc<RefCell<DataBus>>,
}

impl Accumulator {
    pub fn new(data_bus: Rc<RefCell<DataBus>>) -> Self {
        Accumulator {
            register: 0,
            data_bus,
        }
    }

    pub fn write_to_bus(&self) {
        self.data_bus.borrow_mut().write(self.register);
    }

    pub fn read_from_bus(&mut self) {
        self.register = self.data_bus.borrow().read();
    }
}

pub struct InstructionRegister {
    register: u8,
    data_bus: Rc<RefCell<DataBus>>,
}

impl InstructionRegister {
    pub fn new(data_bus: Rc<RefCell<DataBus>>) -> Self {
        InstructionRegister {
            register: 0,
            data_bus,
        }
    }

    pub fn read_from_bus(&mut self) {
        self.register = self.data_bus.borrow().read();
    }

    pub fn set_value(&mut self, value: u8) {
        self.register = value;
    }

    pub fn decode_instruction(&self) -> Instruction {
        let low_nibble = self.register & 0x0F;
        let high_nibble = (self.register & 0xF0) >> 4;

        match low_nibble {
            0x0 => match high_nibble {
                0x0 => Instruction::BRK(AddressingMode::Implied, 1, 7, Penalty::None),
                0x1 => Instruction::BPL(AddressingMode::Relative, 2, 2, Penalty::BranchTaken),
                0x2 => Instruction::JSR(AddressingMode::Absolute, 3, 6, Penalty::None),
                0x3 => Instruction::BMI(AddressingMode::Relative, 2, 2, Penalty::BranchTaken),
                0x4 => Instruction::RTI(AddressingMode::Implied, 1, 6, Penalty::None),
                0x5 => Instruction::BVC(AddressingMode::Relative, 2, 2, Penalty::BranchTaken),
                0x6 => Instruction::RTS(AddressingMode::Implied, 1, 6, Penalty::None),
                0x7 => Instruction::BVS(AddressingMode::Relative, 2, 2, Penalty::BranchTaken),
                0x8 => Instruction::KIL,
                0x9 => Instruction::BCC(AddressingMode::Relative, 2, 2, Penalty::BranchTaken),
                0xA => Instruction::LDY(AddressingMode::Immediate, 2, 2, Penalty::None),
                0xB => Instruction::BCS(AddressingMode::Relative, 2, 2, Penalty::BranchTaken),
                0xC => Instruction::CPY(AddressingMode::Immediate, 2, 2, Penalty::None),
                0xD => Instruction::BNE(AddressingMode::Relative, 2, 2, Penalty::BranchTaken),
                0xE => Instruction::CPX(AddressingMode::Immediate, 2, 2, Penalty::None),
                0xF => Instruction::BEQ(AddressingMode::Relative, 2, 2, Penalty::BranchTaken),
                _ => panic!("nibble should have only 4 bits"),
            },
            0x1 => match high_nibble {
                0x0 => Instruction::ORA(AddressingMode::IndirectX, 2, 6, Penalty::None),
                0x1 => Instruction::ORA(
                    AddressingMode::IndirectY,
                    2,
                    5,
                    Penalty::PageBoundaryCrossed,
                ),
                0x2 => Instruction::AND(AddressingMode::IndirectX, 2, 6, Penalty::None),
                0x3 => Instruction::AND(
                    AddressingMode::IndirectY,
                    2,
                    5,
                    Penalty::PageBoundaryCrossed,
                ),
                0x4 => Instruction::EOR(AddressingMode::IndirectX, 2, 6, Penalty::None),
                0x5 => Instruction::EOR(
                    AddressingMode::IndirectY,
                    2,
                    5,
                    Penalty::PageBoundaryCrossed,
                ),
                0x6 => Instruction::ADC(AddressingMode::IndirectX, 2, 6, Penalty::None),
                0x7 => Instruction::ADC(
                    AddressingMode::IndirectY,
                    2,
                    5,
                    Penalty::PageBoundaryCrossed,
                ),
                0x8 => Instruction::STA(AddressingMode::IndirectX, 2, 6, Penalty::None),
                0x9 => Instruction::STA(AddressingMode::IndirectY, 2, 6, Penalty::None),
                0xA => Instruction::LDA(AddressingMode::IndirectX, 2, 6, Penalty::None),
                0xB => Instruction::LDA(
                    AddressingMode::IndirectY,
                    2,
                    5,
                    Penalty::PageBoundaryCrossed,
                ),
                0xC => Instruction::CMP(AddressingMode::IndirectX, 2, 6, Penalty::None),
                0xD => Instruction::CMP(
                    AddressingMode::IndirectY,
                    2,
                    5,
                    Penalty::PageBoundaryCrossed,
                ),
                0xE => Instruction::SBC(AddressingMode::IndirectX, 2, 6, Penalty::None),
                0xF => Instruction::SBC(
                    AddressingMode::IndirectY,
                    2,
                    5,
                    Penalty::PageBoundaryCrossed,
                ),
                _ => panic!("nibble should have only 4 bits"),
            },
            0x2 => match high_nibble {
                0xA => Instruction::LDX(AddressingMode::Immediate, 2, 2, Penalty::None),
                0x0..=0x9 => Instruction::KIL,
                _ => panic!("nibble should have only 4 bits"),
            },
            0x3 | 0x7 | 0xB | 0xF => Instruction::KIL,
            0x4 => match high_nibble {
                0x2 => Instruction::BIT(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0x8 => Instruction::STY(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0x9 => Instruction::STY(AddressingMode::ZeroPage, 2, 4, Penalty::None),
                0xA => Instruction::LDY(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0xB => Instruction::LDY(AddressingMode::ZeroPage, 2, 4, Penalty::None),
                0xC => Instruction::CPY(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0xE => Instruction::CPX(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0x0 | 0x1 | 0x3..=0x7 | 0xD | 0xF => Instruction::KIL,
                _ => panic!("nibble should have only 4 bits"),
            },
            0x5 => match high_nibble {
                0x0 => Instruction::ORA(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0x1 => Instruction::ORA(AddressingMode::ZeroPageX, 2, 4, Penalty::None),
                0x2 => Instruction::AND(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0x3 => Instruction::AND(AddressingMode::ZeroPageX, 2, 4, Penalty::None),
                0x4 => Instruction::EOR(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0x5 => Instruction::EOR(AddressingMode::ZeroPageX, 2, 4, Penalty::None),
                0x6 => Instruction::ADC(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0x7 => Instruction::ADC(AddressingMode::ZeroPageX, 2, 4, Penalty::None),
                0x8 => Instruction::STA(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0x9 => Instruction::STA(AddressingMode::ZeroPageX, 2, 4, Penalty::None),
                0xA => Instruction::LDA(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0xB => Instruction::LDA(AddressingMode::ZeroPageX, 2, 4, Penalty::None),
                0xC => Instruction::CMP(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0xD => Instruction::CMP(AddressingMode::ZeroPageX, 2, 4, Penalty::None),
                0xE => Instruction::SBC(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0xF => Instruction::SBC(AddressingMode::ZeroPageX, 2, 4, Penalty::None),
                _ => panic!("nibble should have only 4 bits"),
            },
            0x6 => match high_nibble {
                0x0 => Instruction::ASL(AddressingMode::ZeroPage, 2, 5, Penalty::None),
                0x1 => Instruction::ASL(AddressingMode::ZeroPageX, 2, 6, Penalty::None),
                0x2 => Instruction::ROL(AddressingMode::ZeroPage, 2, 5, Penalty::None),
                0x3 => Instruction::ROL(AddressingMode::ZeroPageX, 2, 6, Penalty::None),
                0x4 => Instruction::LSR(AddressingMode::ZeroPage, 2, 5, Penalty::None),
                0x5 => Instruction::LSR(AddressingMode::ZeroPageX, 2, 6, Penalty::None),
                0x6 => Instruction::ROR(AddressingMode::ZeroPage, 2, 5, Penalty::None),
                0x7 => Instruction::ROR(AddressingMode::ZeroPageX, 2, 6, Penalty::None),
                0x8 => Instruction::STX(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0x9 => Instruction::STX(AddressingMode::ZeroPageX, 2, 4, Penalty::None),
                0xA => Instruction::LDX(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0xB => Instruction::LDX(AddressingMode::ZeroPageX, 2, 4, Penalty::None),
                0xC => Instruction::DEC(AddressingMode::ZeroPage, 2, 5, Penalty::None),
                0xD => Instruction::DEC(AddressingMode::ZeroPageX, 2, 6, Penalty::None),
                0xE => Instruction::INC(AddressingMode::ZeroPage, 2, 5, Penalty::None),
                0xF => Instruction::INC(AddressingMode::ZeroPageX, 2, 6, Penalty::None),
                _ => panic!("nibble should have only 4 bits"),
            },
            0x8 => match high_nibble {
                0x0 => Instruction::PHP(AddressingMode::Implied, 1, 3, Penalty::None),
                0x1 => Instruction::CLC(AddressingMode::Implied, 1, 2, Penalty::None),
                0x2 => Instruction::PLP(AddressingMode::Implied, 1, 4, Penalty::None),
                0x3 => Instruction::SEC(AddressingMode::Implied, 1, 2, Penalty::None),
                0x4 => Instruction::PHA(AddressingMode::Implied, 1, 3, Penalty::None),
                0x5 => Instruction::CLI(AddressingMode::Implied, 1, 2, Penalty::None),
                0x6 => Instruction::PLA(AddressingMode::Implied, 1, 4, Penalty::None),
                0x7 => Instruction::SEI(AddressingMode::Implied, 1, 2, Penalty::None),
                0x8 => Instruction::DEY(AddressingMode::Implied, 1, 2, Penalty::None),
                0x9 => Instruction::TYA(AddressingMode::Implied, 1, 2, Penalty::None),
                0xA => Instruction::TAY(AddressingMode::Implied, 1, 2, Penalty::None),
                0xB => Instruction::CLV(AddressingMode::Implied, 1, 2, Penalty::None),
                0xC => Instruction::INY(AddressingMode::Implied, 1, 2, Penalty::None),
                0xD => Instruction::CLD(AddressingMode::Implied, 1, 2, Penalty::None),
                0xE => Instruction::INX(AddressingMode::Implied, 1, 2, Penalty::None),
                0xF => Instruction::SED(AddressingMode::Implied, 1, 2, Penalty::None),
                _ => panic!("nibble should have only 4 bits"),
            },
            0x9 => match high_nibble {
                0x0 => Instruction::ORA(AddressingMode::Immediate, 2, 2, Penalty::None),
                0x1 => Instruction::ORA(
                    AddressingMode::AbsoluteY,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                0x2 => Instruction::AND(AddressingMode::Immediate, 2, 2, Penalty::None),
                0x3 => Instruction::AND(
                    AddressingMode::AbsoluteY,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                0x4 => Instruction::EOR(AddressingMode::Immediate, 2, 2, Penalty::None),
                0x5 => Instruction::EOR(
                    AddressingMode::AbsoluteY,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                0x6 => Instruction::ADC(AddressingMode::Immediate, 2, 2, Penalty::None),
                0x7 => Instruction::ADC(
                    AddressingMode::AbsoluteY,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                0x8 => Instruction::KIL,
                0x9 => Instruction::STA(AddressingMode::AbsoluteY, 3, 5, Penalty::None),
                0xA => Instruction::LDA(AddressingMode::Immediate, 2, 2, Penalty::None),
                0xB => Instruction::LDA(
                    AddressingMode::AbsoluteY,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                0xC => Instruction::CMP(AddressingMode::Immediate, 2, 2, Penalty::None),
                0xD => Instruction::CMP(
                    AddressingMode::AbsoluteY,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                0xE => Instruction::SBC(AddressingMode::Immediate, 2, 2, Penalty::None),
                0xF => Instruction::SBC(
                    AddressingMode::AbsoluteY,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                _ => panic!("nibble should have only 4 bits"),
            },
            0xA => match high_nibble {
                0x0 => Instruction::ASL(AddressingMode::Accumulator, 1, 2, Penalty::None),
                0x2 => Instruction::ROL(AddressingMode::Accumulator, 1, 2, Penalty::None),
                0x4 => Instruction::LSR(AddressingMode::Accumulator, 1, 2, Penalty::None),
                0x6 => Instruction::ROR(AddressingMode::Accumulator, 1, 2, Penalty::None),
                0x8 => Instruction::TXA(AddressingMode::Implied, 1, 2, Penalty::None),
                0x9 => Instruction::TXS(AddressingMode::Implied, 1, 2, Penalty::None),
                0xA => Instruction::TAX(AddressingMode::Implied, 1, 2, Penalty::None),
                0xB => Instruction::TSX(AddressingMode::Implied, 1, 2, Penalty::None),
                0xC => Instruction::DEX(AddressingMode::Implied, 1, 2, Penalty::None),
                0xE => Instruction::NOP(AddressingMode::Implied, 1, 2, Penalty::None),
                0x1 | 0x3 | 0x5 | 0x7 | 0xD | 0xF => Instruction::KIL,
                _ => panic!("nibble should have only 4 bits"),
            },
            0xC => match high_nibble {
                0x2 => Instruction::BIT(AddressingMode::Absolute, 3, 4, Penalty::None),
                0x4 => Instruction::JMP(AddressingMode::Absolute, 3, 3, Penalty::None),
                0x6 => Instruction::JMP(AddressingMode::Indirect, 3, 5, Penalty::None),
                0x8 => Instruction::STY(AddressingMode::Absolute, 3, 4, Penalty::None),
                0xA => Instruction::LDY(AddressingMode::Absolute, 3, 4, Penalty::None),
                0xB => Instruction::LDY(
                    AddressingMode::AbsoluteX,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                0xC => Instruction::CPY(AddressingMode::Absolute, 3, 4, Penalty::None),
                0xE => Instruction::CPX(AddressingMode::Absolute, 3, 4, Penalty::None),
                0x0 | 0x1 | 0x3 | 0x5 | 0x7 | 0x9 | 0xD | 0xF => Instruction::KIL,
                _ => panic!("nibble should have only 4 bits"),
            },
            0xD => match high_nibble {
                0x0 => Instruction::ORA(AddressingMode::Absolute, 3, 4, Penalty::None),
                0x1 => Instruction::ORA(
                    AddressingMode::AbsoluteX,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                0x2 => Instruction::AND(AddressingMode::Absolute, 3, 4, Penalty::None),
                0x3 => Instruction::AND(
                    AddressingMode::AbsoluteX,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                0x4 => Instruction::EOR(AddressingMode::Absolute, 3, 4, Penalty::None),
                0x5 => Instruction::EOR(
                    AddressingMode::AbsoluteX,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                0x6 => Instruction::ADC(AddressingMode::Absolute, 3, 4, Penalty::None),
                0x7 => Instruction::ADC(
                    AddressingMode::AbsoluteX,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                0x8 => Instruction::STA(AddressingMode::Absolute, 3, 4, Penalty::None),
                0x9 => Instruction::STA(
                    AddressingMode::AbsoluteX,
                    3,
                    5,
                    Penalty::PageBoundaryCrossed,
                ),
                0xA => Instruction::LDA(AddressingMode::Absolute, 3, 4, Penalty::None),
                0xB => Instruction::LDA(
                    AddressingMode::AbsoluteX,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                0xC => Instruction::CMP(AddressingMode::Absolute, 3, 4, Penalty::None),
                0xD => Instruction::CMP(
                    AddressingMode::AbsoluteX,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                0xE => Instruction::SBC(AddressingMode::Absolute, 3, 4, Penalty::None),
                0xF => Instruction::SBC(
                    AddressingMode::AbsoluteX,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                _ => panic!("nibble should have only 4 bits"),
            },
            0xE => match high_nibble {
                0x0 => Instruction::ASL(AddressingMode::Absolute, 3, 6, Penalty::None),
                0x1 => Instruction::ASL(AddressingMode::AbsoluteX, 3, 7, Penalty::None),
                0x2 => Instruction::ROL(AddressingMode::Absolute, 3, 6, Penalty::None),
                0x3 => Instruction::ROL(AddressingMode::AbsoluteX, 3, 7, Penalty::None),
                0x4 => Instruction::LSR(AddressingMode::Absolute, 3, 6, Penalty::None),
                0x5 => Instruction::LSR(AddressingMode::AbsoluteX, 3, 7, Penalty::None),
                0x6 => Instruction::ROR(AddressingMode::Absolute, 3, 6, Penalty::None),
                0x7 => Instruction::ROR(AddressingMode::AbsoluteX, 3, 7, Penalty::None),
                0x8 => Instruction::STX(AddressingMode::Absolute, 3, 4, Penalty::None),
                0x9 => Instruction::KIL,
                0xA => Instruction::LDX(AddressingMode::Absolute, 3, 4, Penalty::None),
                0xB => Instruction::LDX(
                    AddressingMode::AbsoluteX,
                    3,
                    4,
                    Penalty::PageBoundaryCrossed,
                ),
                0xC => Instruction::DEC(AddressingMode::Absolute, 3, 6, Penalty::None),
                0xD => Instruction::DEC(AddressingMode::AbsoluteX, 3, 7, Penalty::None),
                0xE => Instruction::INC(AddressingMode::Absolute, 3, 6, Penalty::None),
                0xF => Instruction::INC(AddressingMode::AbsoluteX, 3, 7, Penalty::None),
                _ => panic!("nibble should have only 4 bits"),
            },
            _ => panic!("nibble should have only 4 bits"),
        }
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

#[derive(Debug)]
pub enum Penalty {
    /// Don't add clock cycles
    None,
    /// Add one to the number of clock cycles if page boundary crossed
    PageBoundaryCrossed,
    /// Add one to the number of clock cycles if branch occurs to same page
    /// Add two to the number of clock cycles if branch occurs to different page
    BranchTaken,
}

/// Tuple is (addressing mode, instruction bytes, clock cycles,ClockCycles)
#[derive(Debug)]
pub enum Instruction {
    /// Add Memory to Accumulator with Carry
    ADC(AddressingMode, u32, u32, Penalty),
    /// "AND" Memory with Accumulator
    AND(AddressingMode, u32, u32, Penalty),
    /// Shift Left One Bit (Memory or Accumulator,ClockCycles)
    ASL(AddressingMode, u32, u32, Penalty),

    /// Branch on Carry Clear
    BCC(AddressingMode, u32, u32, Penalty),
    /// Branch on Carry Set
    BCS(AddressingMode, u32, u32, Penalty),
    /// Branch on Result Zero
    BEQ(AddressingMode, u32, u32, Penalty),
    /// Test Bits in Memory with Accumulator
    BIT(AddressingMode, u32, u32, Penalty),
    /// Branch on Result Minus
    BMI(AddressingMode, u32, u32, Penalty),
    /// Branch on Result not Zero
    BNE(AddressingMode, u32, u32, Penalty),
    /// Branch on Result Plus
    BPL(AddressingMode, u32, u32, Penalty),
    /// Force Break
    BRK(AddressingMode, u32, u32, Penalty),
    /// Branch on Overflow Clear
    BVC(AddressingMode, u32, u32, Penalty),
    /// Branch on Overflow Set
    BVS(AddressingMode, u32, u32, Penalty),

    /// Clear Carry Flag
    CLC(AddressingMode, u32, u32, Penalty),
    /// Clear Decimal Mode
    CLD(AddressingMode, u32, u32, Penalty),
    /// Clear Interrupt Disable Bit
    CLI(AddressingMode, u32, u32, Penalty),
    /// Clear Overflow Flag
    CLV(AddressingMode, u32, u32, Penalty),
    /// Compare Memory and Accumulator
    CMP(AddressingMode, u32, u32, Penalty),
    /// Compare Memory and Index X
    CPX(AddressingMode, u32, u32, Penalty),
    /// Compare Memory and Index Y
    CPY(AddressingMode, u32, u32, Penalty),

    /// Decrement Memory by One
    DEC(AddressingMode, u32, u32, Penalty),
    /// Decrement Index X by One
    DEX(AddressingMode, u32, u32, Penalty),
    /// Decrement Index Y by One
    DEY(AddressingMode, u32, u32, Penalty),

    /// "Exclusive-OR" Memory with Accumulator
    EOR(AddressingMode, u32, u32, Penalty),

    /// Increment Memory by One
    INC(AddressingMode, u32, u32, Penalty),
    /// Increment Index X by One
    INX(AddressingMode, u32, u32, Penalty),
    /// Increment Index Y by One
    INY(AddressingMode, u32, u32, Penalty),

    /// Jump to New Location
    JMP(AddressingMode, u32, u32, Penalty),
    /// Jump to New Location Saving Return Address
    JSR(AddressingMode, u32, u32, Penalty),

    /// Load Accumulator with Memory
    LDA(AddressingMode, u32, u32, Penalty),
    /// Load Index X with Memory
    LDX(AddressingMode, u32, u32, Penalty),
    /// Load Index Y with Memory
    LDY(AddressingMode, u32, u32, Penalty),
    /// Shift One Bit Right (Memory or Accumulator,ClockCycles)
    LSR(AddressingMode, u32, u32, Penalty),

    /// No Operation
    NOP(AddressingMode, u32, u32, Penalty),

    /// "OR" Memory with Accumulator
    ORA(AddressingMode, u32, u32, Penalty),

    /// Push Accumulator on Stack
    PHA(AddressingMode, u32, u32, Penalty),
    /// Push Processor Status on Stack
    PHP(AddressingMode, u32, u32, Penalty),
    /// Pull Accumulator from Stack
    PLA(AddressingMode, u32, u32, Penalty),
    /// Pull Processor Status from Stack
    PLP(AddressingMode, u32, u32, Penalty),

    /// Rotate One Bit Left (Memory or Accumulator,ClockCycles)
    ROL(AddressingMode, u32, u32, Penalty),
    /// Rotate One Bit Right (Memory or Accumulator,ClockCycles)
    ROR(AddressingMode, u32, u32, Penalty),
    /// Return from Interrupt
    RTI(AddressingMode, u32, u32, Penalty),
    /// Return from Subroutine
    RTS(AddressingMode, u32, u32, Penalty),

    /// Subtract Memory from Accumulator with Borrow
    SBC(AddressingMode, u32, u32, Penalty),
    /// Set Carry Flag
    SEC(AddressingMode, u32, u32, Penalty),
    /// Set Decimal Mode
    SED(AddressingMode, u32, u32, Penalty),
    /// Set Interrupt Disable Status
    SEI(AddressingMode, u32, u32, Penalty),
    /// Store Accumulator in Memoryj
    STA(AddressingMode, u32, u32, Penalty),
    /// Store Index X in Memory
    STX(AddressingMode, u32, u32, Penalty),
    /// Store Index Y in Memory
    STY(AddressingMode, u32, u32, Penalty),

    /// Transfer Accumulator to Index X
    TAX(AddressingMode, u32, u32, Penalty),
    /// Transfer Accumulator to Index Y
    TAY(AddressingMode, u32, u32, Penalty),
    /// Transfer Stack Pointer to Index X
    TSX(AddressingMode, u32, u32, Penalty),
    /// Transfer Index X to Accumulator
    TXA(AddressingMode, u32, u32, Penalty),
    /// Transfer Index X to Stack Register
    TXS(AddressingMode, u32, u32, Penalty),
    /// Transfer Index Y to Accumulator
    TYA(AddressingMode, u32, u32, Penalty),

    /// Illegal opcode
    KIL,
}

pub struct DataBus {
    bus: u8,
}

impl DataBus {
    pub fn new() -> Self {
        DataBus { bus: 0 }
    }

    pub fn read(&self) -> u8 {
        self.bus
    }

    pub fn write(&mut self, value: u8) {
        self.bus = value;
    }
}

pub struct Alu {
    data: u8,
    data_bus: Rc<RefCell<DataBus>>,
    p: Rc<RefCell<StatusRegister>>,
}

impl Alu {
    pub fn new(data_bus: Rc<RefCell<DataBus>>, p: Rc<RefCell<StatusRegister>>) -> Self {
        Alu {
            data: 0,
            data_bus,
            p,
        }
    }

    fn convert_from_bcd(value: u8) -> u16 {
        let lsd = (value & 0x0F) as u16;
        let msd = ((value & 0xF0) >> 4) as u16;
        msd * 10 + lsd
    }

    fn convert_to_bcd(value: u16) -> u16 {
        let mut bcd = 0;
        let mut value = value;
        for exp in (0..3).rev() {
            let divisor = (10 as u16).pow(exp);
            let digit = value / divisor;
            bcd += digit * (16 as u16).pow(exp);
            value -= digit * divisor;
        }
        bcd
    }

    pub fn add_with_carry(&mut self) {
        let was_negative = self.data & 0x80 == 0x80;

        if self.p.borrow().get_decimal_mode() {
            // Convert currently stored data from BCD to decimal
            let data = Alu::convert_from_bcd(self.data);

            // Convert data on bus from BCD to decimal
            let operand = Alu::convert_from_bcd(self.data_bus.borrow().read());

            // Add in decimal and convert back into BCD
            let bcd = Alu::convert_to_bcd(data + operand + (self.p.borrow().get_carry() as u16));

            // Set flags
            self.p.borrow_mut().set_carry(bcd & 0x100 == 0x100);
            // BCD sets zero flag even if the carry bit is set
            self.p.borrow_mut().set_zero(bcd | 0x00 == 0x00);

            self.data = (bcd & 0xFF) as u8;
        } else {
            let sum = (self.data as u16)
                + (self.data_bus.borrow().read() as u16)
                + (self.p.borrow().get_carry() as u16);

            self.p.borrow_mut().set_carry(sum & 0x100 == 0x100);
            self.p.borrow_mut().set_zero(self.data == 0);

            self.data = (sum & 0xFF) as u8;
        }

        let is_negative = self.data & 0x80 == 0x80;
        self.p.borrow_mut().set_negative(is_negative);
        self.p.borrow_mut().set_overflow(was_negative ^ is_negative);
    }

    pub fn subtract_with_borrow(&mut self) {
        let was_negative = self.data & 0x80 == 0x80;
        let is_negative;

        if self.p.borrow().get_decimal_mode() {
            // Convert currently stored data from BCD to decimal
            let data = Alu::convert_from_bcd(self.data);

            // Convert data on bus from BCD to decimal
            let operand = Alu::convert_from_bcd(self.data_bus.borrow().read());

            // Subtract in decimal
            let sum = data
                .wrapping_sub(operand)
                .wrapping_sub(self.p.borrow().get_carry() as u16);
            // Got a result smaller than 0 (greater than 100)
            // Add 96 to convert to BCD negative
            let sum = if sum > 99 { sum + 96 } else { sum };
            is_negative = sum & 0x100 == 0x100;
            // Convert decimal back into BCD (take lower byte)
            let bcd = Alu::convert_to_bcd(sum & 0xFF);
            let bcd = bcd as u8;

            // Set flags
            // Carry = inverse of borrow
            self.p.borrow_mut().set_carry(sum & 0x100 == 0x100);
            // BCD sets zero flag even if the carry bit is set
            self.p.borrow_mut().set_zero(bcd | 0x00 == 0x00);

            self.data = bcd;
        } else {
            let data = if self.data & 0x80 == 0x80 {
                self.data as u16 | 0xFF00
            } else {
                self.data as u16
            };
            let sum = data
                .wrapping_add(!(self.data_bus.borrow().read() as u16))
                .wrapping_add(self.p.borrow().get_carry() as u16);

            self.data = (sum & 0xFF) as u8;
            // Carry = inverse of borrow
            self.p.borrow_mut().set_carry(sum & 0x100 == 0x100);
            self.p.borrow_mut().set_zero(self.data == 0);

            is_negative = self.data & 0x80 == 0x80;
        }

        self.p.borrow_mut().set_negative(is_negative);
        self.p.borrow_mut().set_overflow(was_negative ^ is_negative);
    }

    pub fn write_to_bus(&self) {
        self.data_bus.borrow_mut().write(self.data);
    }

    pub fn read_from_bus(&mut self) {
        self.data = self.data_bus.borrow().read();
    }
}

pub struct InternalMemory {
    ram: [u8; 0x800],
    data_bus: Rc<RefCell<DataBus>>,
    address_bus: Rc<RefCell<AddressBus>>,
}

impl InternalMemory {
    pub fn new(
        data_bus: Rc<RefCell<DataBus>>,
        address_bus: Rc<RefCell<AddressBus>>,
        memory: Option<&[u8]>,
    ) -> Self {
        let mut mem = InternalMemory {
            ram: [0; 0x800],
            data_bus,
            address_bus,
        };
        if let Some(memory) = memory {
            mem.ram.copy_from_slice(memory);
        }
        mem
    }

    pub fn read(&self) -> u8 {
        let (high, low) = self.address_bus.borrow().read();
        let address = ((high as u16) << 8) | (low as u16);
        self.ram[address as usize]
    }

    pub fn write(&mut self) {
        let (high, low) = self.address_bus.borrow().read();
        let address = ((high as u16) << 8) | (low as u16);
        self.ram[address as usize] = self.data_bus.borrow().read();
    }
}

pub struct Mos6502 {
    internal_ram: InternalMemory,
    /// Accumulator
    a: Accumulator,
    /// ALU
    alu: Alu,
    /// X index register
    x: u8,
    /// Y index register
    y: u8,
    /// Program counter
    pc: Rc<RefCell<ProgramCounter>>,
    /// Stack register
    #[allow(dead_code)]
    s: u8,
    /// Status register
    pub p: Rc<RefCell<StatusRegister>>,
    /// Instruction register
    instruction_register: InstructionRegister,
    data_bus: Rc<RefCell<DataBus>>,
    /// Number of cycles remaining in current instruction
    cycles: u32,
    #[allow(dead_code)]
    output_clock1: bool,
    #[allow(dead_code)]
    output_clock2: bool,
    address_bus: Rc<RefCell<AddressBus>>,
    #[allow(dead_code)]
    ready: bool,
    #[allow(dead_code)]
    not_irq: bool,
    #[allow(dead_code)]
    not_nmi: bool,
    #[allow(dead_code)]
    not_set_overflow: bool,
    #[allow(dead_code)]
    sync: bool,
    #[allow(dead_code)]
    not_reset: bool,
}

enum IndexRegister {
    X,
    Y,
}

impl Mos6502 {
    /// Initializes a new `Mos6502` processor emulator.
    ///
    /// Optionally, provide a `&[u8]` as the starting memory.
    /// The starting memory must be exactly 2,048 bytes long
    /// because it uses `copy_from_slice` under the hood.
    ///
    /// ```
    /// use rustendo_lib::mos6502::Mos6502;
    /// let mut mem = vec![0; 0x800];
    /// // Loads $FF into the accumulator.
    /// mem[0] = 0x69; // ADC
    /// mem[1] = 0xFF; // $FF
    /// let mos6502 = Mos6502::new(Some(&mem));
    /// ```
    pub fn new(memory: Option<&[u8]>) -> Self {
        // Temporarily initializing some fields
        // Will be reinitialized later in the function to point to the correct references.
        let data_bus = Rc::new(RefCell::new(DataBus::new()));
        let address_bus = Rc::new(RefCell::new(AddressBus::new()));
        let pc = Rc::new(RefCell::new(ProgramCounter::new(
            Rc::clone(&data_bus),
            Rc::clone(&address_bus),
        )));
        let p = Rc::new(RefCell::new(StatusRegister::new()));

        Mos6502 {
            internal_ram: InternalMemory::new(
                Rc::clone(&data_bus),
                Rc::clone(&address_bus),
                memory,
            ),
            a: Accumulator::new(Rc::clone(&data_bus)),
            alu: Alu::new(Rc::clone(&data_bus), Rc::clone(&p)),
            instruction_register: InstructionRegister::new(Rc::clone(&data_bus)),
            address_bus,
            x: 0,
            y: 0,
            pc,
            s: 0xFD,
            p,
            data_bus: Rc::clone(&data_bus),
            cycles: 0,
            output_clock1: false,
            output_clock2: false,
            ready: false,
            not_irq: true,
            not_nmi: true,
            not_reset: true,
            not_set_overflow: true,
            sync: false,
        }
    }

    /// Runs the processor for a single clock cycle
    ///
    /// Really, it does everything in one go on the
    /// first clock cycle and then spends the rest of
    /// the time doing nothing.
    ///
    /// Returns true if the instruction is complete.
    ///
    /// ```
    /// use rustendo_lib::mos6502::Mos6502;
    /// let mut mos6502 = Mos6502::new(None);
    /// // Runs a single instruction to completion.
    /// while mos6502.clock() { }
    /// ```
    pub fn clock(&mut self) -> bool {
        if self.cycles == 0 {
            self.read_instruction();
            self.execute_instruction();
        }

        self.cycles -= 1;
        self.cycles == 0
    }

    fn fetch_next_byte(&mut self) -> u8 {
        self.pc.borrow_mut().increment();
        self.pc.borrow().write_to_address_bus();
        self.internal_ram.read()
    }

    fn absolute_indexed_addressing(&mut self, index: IndexRegister) {
        let address_low = self.fetch_next_byte();
        let mut address_high = self.fetch_next_byte();
        let register: &mut u8 = match index {
            IndexRegister::X => &mut self.x,
            IndexRegister::Y => &mut self.y,
        };
        let (new_index, carry) = register.overflowing_add(address_low);
        *register = new_index;
        if carry {
            // a carry occurred (page boundary crossed), need to add one
            // to high byte of address and use additional cycle
            address_high += 1;
            self.cycles += 1;
        }
        self.address_bus
            .borrow_mut()
            .write(address_high, address_low);
    }

    fn do_addressing_mode(&mut self, mode: AddressingMode) {
        match mode {
            AddressingMode::Absolute => {
                let address_low = self.fetch_next_byte();
                let address_high = self.fetch_next_byte();
                self.address_bus
                    .borrow_mut()
                    .write(address_high, address_low);
            }
            AddressingMode::AbsoluteX => self.absolute_indexed_addressing(IndexRegister::X),
            AddressingMode::AbsoluteY => self.absolute_indexed_addressing(IndexRegister::Y),
            AddressingMode::Accumulator => {
                self.data_bus.borrow_mut().write(self.a.register);
            }
            AddressingMode::Immediate => {
                let value = self.fetch_next_byte();
                self.data_bus.borrow_mut().write(value);
            }
            AddressingMode::Implied => unimplemented!("{:?} unimplemented", mode),
            AddressingMode::Indirect => {
                let zero_page_offset = self.fetch_next_byte();
                self.address_bus.borrow_mut().write(0, zero_page_offset);
                let address_low = self.internal_ram.read();
                self.address_bus.borrow_mut().write(0, zero_page_offset + 1);
                let address_high = self.internal_ram.read();
                self.address_bus
                    .borrow_mut()
                    .write(address_high, address_low);
            }
            AddressingMode::IndirectX => {
                // Indexed indirect addressing with register X
                let zero_page_offset = self.fetch_next_byte();
                let zero_page_offset = zero_page_offset.wrapping_add(self.x);
                self.address_bus.borrow_mut().write(0, zero_page_offset);
                let address_low = self.internal_ram.read();
                let zero_page_offset = zero_page_offset.wrapping_add(1);
                self.address_bus.borrow_mut().write(0, zero_page_offset);
                let address_high = self.internal_ram.read();
                self.address_bus
                    .borrow_mut()
                    .write(address_high, address_low);
            }
            AddressingMode::IndirectY => {
                // Indirect indexed addressing with register Y
                let zero_page_offset = self.fetch_next_byte();
                self.address_bus.borrow_mut().write(0, zero_page_offset);
                let address_low = self.internal_ram.read();
                let zero_page_offset = zero_page_offset.wrapping_add(1);
                self.address_bus.borrow_mut().write(0, zero_page_offset);
                let mut address_high = self.internal_ram.read();
                let (address_low, carry) = address_low.overflowing_add(self.y);
                if carry {
                    // a carry occurred (page boundary crossed), need to add one
                    // to high byte of address and use additional cycle
                    address_high += 1;
                    self.cycles += 1;
                }
                self.address_bus
                    .borrow_mut()
                    .write(address_high, address_low);
            }
            AddressingMode::Relative => {
                self.p.borrow_mut().set_carry(false);
                let offset = self.fetch_next_byte();
                let offset_negative = offset & 0x80 == 0x80;

                // PCL + offset -> PCL
                self.data_bus.borrow_mut().write(self.pc.borrow().low());
                self.alu.read_from_bus();
                self.data_bus.borrow_mut().write(offset);
                self.alu.read_from_bus();
                self.alu.add_with_carry();
                self.alu.write_to_bus();
                self.pc.borrow_mut().read_low_from_bus();

                // If the offset was negative, we expect a carry
                // when no page boundary is crossed
                if offset_negative && self.p.borrow().get_carry() {
                    return;
                }

                // If the offset was positive, we expect no carry
                // when no page boundary is crossed.
                if !offset_negative && !self.p.borrow().get_carry() {
                    return;
                }

                // Page boundary crossed, additional cycle needed
                // and we must calculate the new PCH.
                self.cycles += 1;

                // PCH + 0 + carry -> PCH
                self.data_bus.borrow().read();
                self.data_bus.borrow_mut().write(0);
                self.alu.read_from_bus();
                self.data_bus.borrow_mut().write(self.pc.borrow().high());
                self.alu.read_from_bus();
                self.alu.add_with_carry();
                self.pc.borrow_mut().read_high_from_bus();
            }
            AddressingMode::ZeroPage => {
                let zero_page_offset = self.fetch_next_byte();
                self.address_bus.borrow_mut().write(0, zero_page_offset);
            }
            AddressingMode::ZeroPageX => {
                let zero_page_offset = self.fetch_next_byte();
                let zero_page_offset = zero_page_offset.wrapping_add(self.x);
                self.address_bus.borrow_mut().write(0, zero_page_offset);
            }
            AddressingMode::ZeroPageY => {
                let zero_page_offset = self.fetch_next_byte();
                let zero_page_offset = zero_page_offset.wrapping_add(self.y);
                self.address_bus.borrow_mut().write(0, zero_page_offset);
            }
        }
    }

    fn read_instruction(&mut self) {
        self.pc.borrow().write_to_address_bus();
        let next_instruction = self.internal_ram.read();
        self.data_bus.borrow_mut().write(next_instruction);
        self.instruction_register.read_from_bus();
    }

    fn execute_instruction(&mut self) {
        let instruction = self.instruction_register.decode_instruction();
        match instruction {
            Instruction::ADC(mode, _, cycles, _) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.alu.add_with_carry();
                self.alu.write_to_bus();
                self.a.read_from_bus();
            }
            Instruction::AND(mode, _, cycles, _) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                let operand = self.data_bus.borrow().read();
                let result = self.a.register & operand;
                self.data_bus.borrow_mut().write(result);
                self.a.read_from_bus();
                self.p.borrow_mut().set_zero(result == 0);
                self.p.borrow_mut().set_negative(result & 0x80 == 0x80);
            }
            Instruction::ASL(mode, _, cycles, _) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                let operand = self.data_bus.borrow().read();
                let result = operand << 1;
                self.data_bus.borrow_mut().write(result);

                // The instruction does mot affect the overflow bit, Sets N equal to the
                // result bit 7 (bit 6 in the input), sets Z flag if the result is equal to
                // 0, otherwise resets Z and stores the input bit 7 in the carry flag.

                self.p.borrow_mut().set_negative(result & 0x80 == 0x80);

                if result == 0 {
                    self.p.borrow_mut().set_zero(true);
                } else {
                    self.p.borrow_mut().set_carry(operand & 0x80 == 0x80);
                }

                if mode == AddressingMode::Accumulator {
                    self.a.read_from_bus();
                } else {
                    self.internal_ram.write();
                }
            }
            Instruction::BCC(mode, _, cycles, _) => {
                self.cycles = cycles;

                if !self.p.borrow().get_carry() {
                    // Branch taken, add an additional cycle.
                    self.cycles += 1;
                    self.do_addressing_mode(mode);
                }
            }
            Instruction::CLC(_, _, cycles, _) => {
                self.cycles = cycles;
                self.p.borrow_mut().set_carry(false);
            }
            Instruction::CLD(_, _, cycles, _) => {
                self.cycles = cycles;
                self.p.borrow_mut().set_decimal_mode(false);
            }
            Instruction::SBC(mode, _, cycles, _) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.alu.subtract_with_borrow();
                self.alu.write_to_bus();
                self.a.read_from_bus();
            }
            Instruction::SEC(_, _, cycles, _) => {
                self.cycles = cycles;
                self.p.borrow_mut().set_carry(true);
            }
            Instruction::SED(_, _, cycles, _) => {
                self.cycles = cycles;
                self.p.borrow_mut().set_decimal_mode(true);
            }
            Instruction::STA(mode, _, cycles, _) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.a.write_to_bus();
                self.internal_ram.write();
            }
            Instruction::BRK(_, _, cycles, _) => {
                // For now, just set cycles so that this instruction works for doctests
                self.cycles = cycles;
            }
            instruction => unimplemented!("{:?} instruction is unimplemented", instruction),
        }

        self.pc.borrow_mut().increment();
    }

    /// Get memory at a particular address.
    ///
    /// Useful for testing. Resets internal address bus register
    /// to its old state.
    ///
    /// ```
    /// use rustendo_lib::mos6502::Mos6502;
    /// let mut mos6502 = Mos6502::new(None);
    /// let mem = mos6502.read_memory_at_address(0);
    /// assert_eq!(mem, 0x0);
    /// ```
    pub fn read_memory_at_address(&mut self, address: u16) -> u8 {
        let old_address = self.address_bus.borrow().read();
        self.address_bus.borrow_mut().write_wide(address);
        let mem = self.internal_ram.read();
        self.address_bus
            .borrow_mut()
            .write(old_address.0, old_address.1);
        mem
    }
}
