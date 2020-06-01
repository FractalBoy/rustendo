use crate::bus::Bus;
use std::cell::RefCell;
use std::rc::Rc;

struct DataBus {
    data: u8,
    bus: Rc<RefCell<Bus>>,
}

impl DataBus {
    pub fn new(bus: &Rc<RefCell<Bus>>) -> Self {
        DataBus {
            data: 0,
            bus: Rc::clone(bus),
        }
    }

    pub fn read(&self) -> u8 {
        self.data
    }

    pub fn write(&mut self, data: u8) {
        self.data = data
    }

    pub fn write_to_bus(&self) {
        self.bus.borrow_mut().write_data(self.data);
        self.bus.borrow_mut().write();
    }

    pub fn write_directly_to_bus(&mut self, data: u8) {
        self.write(data);
        self.write_to_bus();
    }

    pub fn read_from_bus(&mut self) {
        self.data = self.bus.borrow_mut().read();
    }

    pub fn read_directly_from_bus(&mut self) -> u8 {
        self.read_from_bus();
        self.read()
    }
}

struct AddressBus {
    address_high: u8,
    address_low: u8,
    bus: Rc<RefCell<Bus>>,
}

impl AddressBus {
    pub fn new(bus: &Rc<RefCell<Bus>>) -> Self {
        AddressBus {
            address_high: 0,
            address_low: 0,
            bus: Rc::clone(bus),
        }
    }

    pub fn write(&mut self, address_high: u8, address_low: u8) {
        self.address_high = address_high;
        self.address_low = address_low;
    }

    pub fn write_directly_to_bus(&mut self, address_high: u8, address_low: u8) {
        self.write(address_high, address_low);
        self.write_to_bus();
    }

    pub fn write_to_bus(&self) {
        self.bus
            .borrow_mut()
            .write_address(self.address_high, self.address_low);
    }
}

struct ProgramCounter {
    pch: u8,
    pcl: u8,
    data_bus: Rc<RefCell<DataBus>>,
    address_bus: Rc<RefCell<AddressBus>>,
}

impl ProgramCounter {
    pub fn new(data_bus: &Rc<RefCell<DataBus>>, address_bus: &Rc<RefCell<AddressBus>>) -> Self {
        ProgramCounter {
            pch: 0,
            pcl: 0,
            data_bus: Rc::clone(data_bus),
            address_bus: Rc::clone(address_bus),
        }
    }

    pub fn write_to_address_bus(&self) {
        self.address_bus
            .borrow_mut()
            .write_directly_to_bus(self.pch, self.pcl);
    }

    pub fn read_high_from_data_bus(&mut self) {
        self.pch = self.data_bus.borrow().read();
    }

    pub fn read_low_from_data_bus(&mut self) {
        self.pcl = self.data_bus.borrow().read();
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

struct StatusRegister {
    carry: bool,
    zero: bool,
    irq_disable: bool,
    decimal_mode: bool,
    brk_command: bool,
    overflow: bool,
    #[allow(dead_code)]
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
            always_one: true,
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

    pub fn set_decimal_mode(&mut self, decimal_mode: bool) {
        self.decimal_mode = decimal_mode
    }

    pub fn get_decimal_mode(&self) -> bool {
        self.decimal_mode
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

struct InstructionRegister {
    data: u8,
    data_bus: Rc<RefCell<DataBus>>,
}

impl InstructionRegister {
    pub fn new(data_bus: &Rc<RefCell<DataBus>>) -> Self {
        InstructionRegister {
            data: 0,
            data_bus: Rc::clone(data_bus),
        }
    }

    pub fn read_from_bus(&mut self) {
        self.data = self.data_bus.borrow().read();
    }

    pub fn decode_instruction(&self) -> Instruction {
        let low_nibble = self.data & 0x0F;
        let high_nibble = (self.data & 0xF0) >> 4;

        match low_nibble {
            0x0 => match high_nibble {
                // BRK is a 2 byte instruction, despite 6502 documentation.
                // That is, the next instruction is at PC + 2
                0x0 => Instruction::BRK(AddressingMode::Implied, 2, 7, Penalty::None),
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
                0x9 => Instruction::STX(AddressingMode::ZeroPageY, 2, 4, Penalty::None),
                0xA => Instruction::LDX(AddressingMode::ZeroPage, 2, 3, Penalty::None),
                0xB => Instruction::LDX(AddressingMode::ZeroPageY, 2, 4, Penalty::None),
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

struct Accumulator {
    data: u8,
    data_bus: Rc<RefCell<DataBus>>,
}

impl Accumulator {
    pub fn new(data_bus: &Rc<RefCell<DataBus>>) -> Self {
        Accumulator {
            data: 0,
            data_bus: Rc::clone(data_bus),
        }
    }

    pub fn write_to_bus(&self) {
        self.data_bus.borrow_mut().write(self.data);
    }

    pub fn read_from_bus(&mut self) {
        self.data = self.data_bus.borrow().read();
    }

    pub fn write(&mut self, data: u8) {
        self.data = data;
    }

    pub fn read(&self) -> u8 {
        self.data
    }
}

struct Alu {
    data_bus: Rc<RefCell<DataBus>>,
    accumulator: Rc<RefCell<Accumulator>>,
}

impl Alu {
    pub fn new(data_bus: &Rc<RefCell<DataBus>>, accumulator: &Rc<RefCell<Accumulator>>) -> Self {
        Alu {
            data_bus: Rc::clone(data_bus),
            accumulator: Rc::clone(accumulator),
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

    pub fn add_with_carry(&mut self, p: &mut StatusRegister) {
        let accumulator_data = self.accumulator.borrow().read();
        let bus_data = self.data_bus.borrow().read();

        let was_negative = accumulator_data & 0x80 == 0x80;
        let sum;

        if p.get_decimal_mode() {
            // Convert currently stored data from BCD to decimal
            let data = Alu::convert_from_bcd(accumulator_data);

            // Convert data on bus from BCD to decimal
            let operand = Alu::convert_from_bcd(bus_data);

            // Add in decimal and convert back into BCD
            let bcd = Alu::convert_to_bcd(data + operand + (p.get_carry() as u16));

            // Set flags
            p.set_carry(bcd & 0x100 == 0x100);
            // BCD sets zero flag even if the carry bit is set
            p.set_zero(bcd | 0x00 == 0x00);
            sum = (bcd & 0xFF) as u8;
        } else {
            let bin = (accumulator_data as u16)
                .wrapping_add(bus_data as u16)
                .wrapping_add(p.get_carry() as u16);

            p.set_carry(bin & 0x100 == 0x100);

            sum = (bin & 0xFF) as u8;
            p.set_zero(sum == 0);
        }

        self.accumulator.borrow_mut().write(sum);
        let is_negative = sum & 0x80 == 0x80;
        p.set_negative(is_negative);
        p.set_overflow(was_negative ^ is_negative);
    }

    pub fn subtract_with_borrow(&mut self, p: &mut StatusRegister) {
        let accumulator_data = self.accumulator.borrow().read();
        let bus_data = self.data_bus.borrow().read();

        let was_negative = accumulator_data & 0x80 == 0x80;
        let is_negative;
        let sum;

        if p.get_decimal_mode() {
            // Convert currently stored data from BCD to decimal
            let data = Alu::convert_from_bcd(accumulator_data);

            // Convert data on bus from BCD to decimal
            let operand = Alu::convert_from_bcd(bus_data);

            // Subtract in decimal
            let dec = (data as i16) - (operand as i16) - ((!p.get_carry()) as i16);
            let dec = if dec < 0 { dec + 100 } else { dec };
            let dec = dec as u16;
            is_negative = dec & 0x100 == 0x100;
            // Convert decimal back into BCD (take lower byte)
            let bcd = Alu::convert_to_bcd((dec & 0xFF) as u16);
            let bcd = bcd as u8;

            // Set flags
            // Carry = inverse of borrow
            p.set_carry(dec & 0x100 == 0x100);
            // BCD sets zero flag even if the carry bit is set
            p.set_zero(bcd | 0x00 == 0x00);

            sum = (bcd & 0xFF) as u8;
        } else {
            let bin = (accumulator_data as u16)
                .wrapping_add((!bus_data) as u16)
                .wrapping_add(p.get_carry() as u16);

            // Carry = inverse of borrow
            p.set_carry(bin & 0x100 == 0x100);

            sum = (bin & 0xFF) as u8;
            p.set_zero(sum == 0);

            is_negative = sum & 0x80 == 0x80;
        }

        self.accumulator.borrow_mut().write(sum);
        p.set_negative(is_negative);
        p.set_overflow(was_negative ^ is_negative);
    }
}

pub struct Mos6502 {
    /// Accumulator
    a: Rc<RefCell<Accumulator>>,
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
    p: StatusRegister,
    /// Instruction register
    instruction_register: InstructionRegister,
    /// Internal data bus buffer
    data_bus: Rc<RefCell<DataBus>>,
    /// Internal address bus
    address_bus: Rc<RefCell<AddressBus>>,
    /// Number of cycles remaining in current instruction
    cycles: u32,
    #[allow(dead_code)]
    output_clock1: bool,
    #[allow(dead_code)]
    output_clock2: bool,
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
    pub fn new(bus: &Rc<RefCell<Bus>>) -> Self {
        let data_bus = DataBus::new(bus);
        let data_bus = Rc::new(RefCell::new(data_bus));

        let address_bus = AddressBus::new(bus);
        let address_bus = Rc::new(RefCell::new(address_bus));

        let a = Accumulator::new(&data_bus);
        let a = Rc::new(RefCell::new(a));

        let pc = ProgramCounter::new(&data_bus, &address_bus);
        let pc = Rc::new(RefCell::new(pc));

        let p = StatusRegister::new();

        let alu = Alu::new(&data_bus, &a);
        let instruction_register = InstructionRegister::new(&data_bus);

        Mos6502 {
            a,
            alu,
            instruction_register,
            x: 0,
            y: 0,
            pc,
            s: 0xFD,
            p,
            data_bus,
            address_bus,
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

    /// Runs the processor for a single clock cycle.
    ///
    /// Really, it does everything in one go on the
    /// first clock cycle and then spends the rest of
    /// the time doing nothing.
    ///
    /// Returns true if the instruction is complete.
    pub fn clock(&mut self) -> bool {
        if self.cycles == 0 {
            self.read_instruction();
            self.execute_instruction();
        }

        self.cycles -= 1;
        self.cycles == 0
    }

    fn fetch_next_byte(&mut self) -> u8 {
        let mut pc = self.pc.borrow_mut();
        pc.increment();
        pc.write_to_address_bus();
        self.address_bus.borrow().write_to_bus();
        self.data_bus.borrow_mut().read_directly_from_bus()
    }

    fn absolute_indexed_addressing(&mut self, index: IndexRegister) {
        let address_low = self.fetch_next_byte();
        let address_high = self.fetch_next_byte();
        let register: &mut u8 = match index {
            IndexRegister::X => &mut self.x,
            IndexRegister::Y => &mut self.y,
        };
        let (new_index, carry) = register.overflowing_add(address_low);
        *register = new_index;
        let address_high = if carry {
            // a carry occurred (page boundary crossed), need to add one
            // to high byte of address and use additional cycle
            self.cycles += 1;
            address_high + 1
        } else {
            address_high
        };

        self.address_bus
            .borrow_mut()
            .write_directly_to_bus(address_high, address_low);
    }

    fn do_addressing_mode(&mut self, mode: AddressingMode) {
        match mode {
            AddressingMode::Absolute => {
                let address_low = self.fetch_next_byte();
                let address_high = self.fetch_next_byte();
                self.address_bus
                    .borrow_mut()
                    .write_directly_to_bus(address_high, address_low);
            }
            AddressingMode::AbsoluteX => self.absolute_indexed_addressing(IndexRegister::X),
            AddressingMode::AbsoluteY => self.absolute_indexed_addressing(IndexRegister::Y),
            AddressingMode::Accumulator => {
                self.data_bus.borrow_mut().write(self.a.borrow().read());
            }
            AddressingMode::Immediate => {
                let value = self.fetch_next_byte();
                self.data_bus.borrow_mut().write(value);
            }
            AddressingMode::Implied => unimplemented!("{:?} unimplemented", mode),
            AddressingMode::Indirect => {
                let zero_page_offset = self.fetch_next_byte();
                let mut address_bus = self.address_bus.borrow_mut();
                let mut data_bus = self.data_bus.borrow_mut();
                address_bus.write_directly_to_bus(0, zero_page_offset);
                let address_low = data_bus.read_directly_from_bus();
                address_bus.write_directly_to_bus(0, zero_page_offset + 1);
                let address_high = data_bus.read_directly_from_bus();
                address_bus.write_directly_to_bus(address_high, address_low);
            }
            AddressingMode::IndirectX => {
                // Indexed indirect addressing with register X
                let zero_page_offset = self.fetch_next_byte();
                let mut address_bus = self.address_bus.borrow_mut();
                let mut data_bus = self.data_bus.borrow_mut();
                let zero_page_offset = zero_page_offset.wrapping_add(self.x);
                address_bus.write_directly_to_bus(0, zero_page_offset);
                let address_low = data_bus.read_directly_from_bus();
                let zero_page_offset = zero_page_offset.wrapping_add(1);
                address_bus.write_directly_to_bus(0, zero_page_offset);
                let address_high = data_bus.read_directly_from_bus();
                address_bus.write(address_high, address_low);
            }
            AddressingMode::IndirectY => {
                // Indirect indexed addressing with register Y
                let zero_page_offset = self.fetch_next_byte();
                let mut address_bus = self.address_bus.borrow_mut();
                let mut data_bus = self.data_bus.borrow_mut();
                address_bus.write_directly_to_bus(0, zero_page_offset);
                let address_low = data_bus.read_directly_from_bus();
                let zero_page_offset = zero_page_offset.wrapping_add(1);
                address_bus.write_directly_to_bus(0, zero_page_offset);
                let address_high = data_bus.read_directly_from_bus();
                let (address_low, carry) = address_low.overflowing_add(self.y);
                let address_high = if carry {
                    // a carry occurred (page boundary crossed), need to add one
                    // to high byte of address and use additional cycle
                    self.cycles += 1;
                    address_high + 1
                } else {
                    address_high
                };
                address_bus.write(address_high, address_low);
            }
            AddressingMode::Relative => {
                let offset = self.fetch_next_byte();
                let offset_negative = offset & 0x80 == 0x80;

                let mut pc = self.pc.borrow_mut();

                // PCL + offset -> PCL
                let (pcl, carry) = pc.read_low().overflowing_add(offset);
                self.data_bus.borrow_mut().write(pcl);
                pc.read_low_from_data_bus();

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
                self.cycles += 1;

                // PCH + 0 + carry -> PCH
                let pch = pc.read_high().wrapping_add(1);
                self.data_bus.borrow_mut().write(pch);
                pc.read_high_from_data_bus();
            }
            AddressingMode::ZeroPage => {
                let zero_page_offset = self.fetch_next_byte();
                self.address_bus
                    .borrow_mut()
                    .write_directly_to_bus(0, zero_page_offset);
            }
            AddressingMode::ZeroPageX => {
                let zero_page_offset = self.fetch_next_byte();
                let zero_page_offset = zero_page_offset.wrapping_add(self.x);
                self.address_bus
                    .borrow_mut()
                    .write_directly_to_bus(0, zero_page_offset);
            }
            AddressingMode::ZeroPageY => {
                let zero_page_offset = self.fetch_next_byte();
                let zero_page_offset = zero_page_offset.wrapping_add(self.y);
                self.address_bus
                    .borrow_mut()
                    .write_directly_to_bus(0, zero_page_offset);
            }
        }
    }

    fn branch(&mut self, branch: bool, mode: AddressingMode, cycles: u32) {
        self.cycles = cycles;

        if branch {
            // Branch taken, add an additional cycle.
            self.cycles += 1;
            self.do_addressing_mode(mode);
        } else {
            // Branch not taken, retrieve parameter and discard
            self.fetch_next_byte();
        }
    }

    fn read_instruction(&mut self) {
        self.pc.borrow().write_to_address_bus();
        self.data_bus.borrow_mut().read_from_bus();
        self.instruction_register.read_from_bus();
    }

    fn execute_instruction(&mut self) {
        let instruction = self.instruction_register.decode_instruction();
        match instruction {
            Instruction::ADC(mode, _, cycles, _) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.alu.add_with_carry(&mut self.p);
            }
            Instruction::AND(mode, _, cycles, _) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                let result;
                {
                    let mut data_bus = self.data_bus.borrow_mut();
                    let operand = data_bus.read();
                    result = self.a.borrow().read() & operand;
                    data_bus.write(result);
                }
                self.a.borrow_mut().read_from_bus();
                self.p.set_zero(result == 0);
                self.p.set_negative(result & 0x80 == 0x80);
            }
            Instruction::ASL(mode, _, cycles, _) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                let operand = self.data_bus.borrow().read();
                let result = operand << 1;
                self.data_bus.borrow_mut().write(result);

                // The instruction does not affect the overflow bit, Sets N equal to the
                // result bit 7 (bit 6 in the input), sets Z flag if the result is equal to
                // 0, otherwise resets Z and stores the input bit 7 in the carry flag.

                {
                    self.p.set_negative(result & 0x80 == 0x80);

                    if result == 0 {
                        self.p.set_zero(true);
                    } else {
                        self.p.set_carry(operand & 0x80 == 0x80);
                    }
                }

                if mode == AddressingMode::Accumulator {
                    self.a.borrow_mut().read_from_bus();
                } else {
                    self.data_bus.borrow().write_to_bus();
                }
            }
            Instruction::BCC(mode, _, cycles, _) => self.branch(!self.p.get_carry(), mode, cycles),
            Instruction::BCS(mode, _, cycles, _) => self.branch(self.p.get_carry(), mode, cycles),
            Instruction::BEQ(mode, _, cycles, _) => self.branch(self.p.get_zero(), mode, cycles),
            Instruction::BIT(mode, _, cycles, _) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                let operand = self.data_bus.borrow().read();
                self.p.set_negative(operand & 0x80 == 0x80);
                self.p.set_overflow(operand * 0x40 == 0x40);
                self.p.set_zero(operand & self.a.borrow().read() == 0);
            }
            Instruction::BMI(mode, _, cycles, _) => {
                self.branch(self.p.get_negative(), mode, cycles)
            }
            Instruction::BNE(mode, _, cycles, _) => self.branch(!self.p.get_zero(), mode, cycles),
            Instruction::BPL(mode, _, cycles, _) => {
                self.branch(!self.p.get_negative(), mode, cycles)
            }
            Instruction::BRK(_, _, cycles, _) => {
                self.cycles = cycles;

                // Padding byte, ignored.
                self.fetch_next_byte();

                let mut address_bus = self.address_bus.borrow_mut();
                let pc = self.pc.borrow();
                address_bus.write_directly_to_bus(0, self.s);
                self.data_bus.borrow_mut().write_directly_to_bus(pc.read_high());
                address_bus.write_directly_to_bus(0, self.s - 1);
                self.data_bus.borrow_mut().write_directly_to_bus(pc.read_low());
                address_bus.write_directly_to_bus(0, self.s - 2);
                self.data_bus.borrow_mut().write_directly_to_bus(self.p.get());

                // Data unused, just putting this here for reference
                address_bus.write_directly_to_bus(0xFF, 0xFE);
                self.data_bus.borrow_mut().read_directly_from_bus();
                address_bus.write_directly_to_bus(0xFF, 0xFF);
                self.data_bus.borrow_mut().read_directly_from_bus();
            }
            Instruction::CLC(_, _, cycles, _) => {
                self.cycles = cycles;
                self.p.set_carry(false);
            }
            Instruction::CLD(_, _, cycles, _) => {
                self.cycles = cycles;
                self.p.set_decimal_mode(false);
            }
            Instruction::NOP(_, _, cycles, _) => self.cycles = cycles,
            Instruction::SBC(mode, _, cycles, _) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.alu.subtract_with_borrow(&mut self.p);
            }
            Instruction::SEC(_, _, cycles, _) => {
                self.cycles = cycles;
                self.p.set_carry(true);
            }
            Instruction::SED(_, _, cycles, _) => {
                self.cycles = cycles;
                self.p.set_decimal_mode(true);
            }
            Instruction::STA(mode, _, cycles, _) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.a.borrow().write_to_bus();
                self.data_bus.borrow().write_to_bus();
            }
            instruction => unimplemented!("{:?} instruction is unimplemented", instruction),
        }

        self.pc.borrow_mut().increment();
    }
}
