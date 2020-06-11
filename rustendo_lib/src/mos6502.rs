use crate::cpu_bus::Bus;
use std::cell::RefCell;
use std::rc::Rc;

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
        self.address_bus.borrow_mut().write(self.pch, self.pcl);
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
    pub carry: bool,
    pub zero: bool,
    pub irq_disable: bool,
    pub decimal_mode: bool,
    pub brk_command: bool,
    pub overflow: bool,
    always_one: bool,
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
                _ => panic!("nibble should have only 4 bits"),
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
                _ => panic!("nibble should have only 4 bits"),
            },
            0x2 => match high_nibble {
                0xA => Instruction::LDX(AddressingMode::Immediate, 2, 2),
                0x0..=0x9 => Instruction::KIL,
                _ => panic!("nibble should have only 4 bits"),
            },
            0x3 | 0x7 | 0xB | 0xF => Instruction::KIL,
            0x4 => match high_nibble {
                0x2 => Instruction::BIT(AddressingMode::ZeroPage, 2, 3),
                0x8 => Instruction::STY(AddressingMode::ZeroPage, 2, 3),
                0x9 => Instruction::STY(AddressingMode::ZeroPage, 2, 4),
                0xA => Instruction::LDY(AddressingMode::ZeroPage, 2, 3),
                0xB => Instruction::LDY(AddressingMode::ZeroPage, 2, 4),
                0xC => Instruction::CPY(AddressingMode::ZeroPage, 2, 3),
                0xE => Instruction::CPX(AddressingMode::ZeroPage, 2, 3),
                0x0 | 0x1 | 0x3..=0x7 | 0xD | 0xF => Instruction::KIL,
                _ => panic!("nibble should have only 4 bits"),
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
                _ => panic!("nibble should have only 4 bits"),
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
                _ => panic!("nibble should have only 4 bits"),
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
                _ => panic!("nibble should have only 4 bits"),
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
                _ => panic!("nibble should have only 4 bits"),
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
                _ => panic!("nibble should have only 4 bits"),
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
                _ => panic!("nibble should have only 4 bits"),
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
                _ => panic!("nibble should have only 4 bits"),
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
                0xB => Instruction::LDX(AddressingMode::AbsoluteX, 3, 4),
                0xC => Instruction::DEC(AddressingMode::Absolute, 3, 6),
                0xD => Instruction::DEC(AddressingMode::AbsoluteX, 3, 7),
                0xE => Instruction::INC(AddressingMode::Absolute, 3, 6),
                0xF => Instruction::INC(AddressingMode::AbsoluteX, 3, 7),
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

    pub fn add_with_carry(&mut self, p: &mut StatusRegister) {
        let accumulator_data = self.accumulator.borrow().read();
        let bus_data = self.data_bus.borrow().read();

        let was_negative = accumulator_data & 0x80 == 0x80;
        let sum;

        if p.decimal_mode {
            // Add the one's digit and the borrow.
            let mut low = (accumulator_data & 0x0F)
                .wrapping_add(bus_data & 0x0F)
                .wrapping_add(p.carry as u8);
            // Add the ten's digit (shift to one's place and mask)
            let mut high = ((accumulator_data >> 4) & 0x0F).wrapping_add((bus_data >> 4) & 0x0F);
            p.carry = false;

            // If the one's digit is greater than nine, it's not valid decimal.
            // Adding 6 and taking the low nibble will fix this, since it causes
            // A -> F to wrap around back to the range 0 -> 5.
            if low > 9 {
                let diff = low - 10;
                low = low - diff;
                high = high + diff;
            }
            if high > 9 {
                p.carry = true;
                high -= 10;
            }

            // Rebuild the result
            sum = (high << 4) | low;

            p.zero = sum == 0;
        } else {
            let bin = (accumulator_data as u16)
                .wrapping_add(bus_data as u16)
                .wrapping_add(p.carry as u16);

            p.carry = bin & 0x100 == 0x100;

            sum = (bin & 0xFF) as u8;
            p.zero = sum == 0;
        }

        self.accumulator.borrow_mut().write(sum);
        p.negative = sum & 0x80 == 0x80;
        p.overflow = was_negative ^ p.negative;
    }

    pub fn subtract_with_borrow(&mut self, p: &mut StatusRegister) {
        let accumulator_data = self.accumulator.borrow().read();
        let bus_data = self.data_bus.borrow().read();

        let was_negative = accumulator_data & 0x80 == 0x80;
        let is_negative;
        let sum;

        if p.decimal_mode {
            // Subtract the one's digit and the borrow.
            let mut low = (accumulator_data & 0x0F)
                .wrapping_sub(bus_data & 0x0F)
                .wrapping_sub(!p.carry as u8);
            // Subtract the ten's digit (shift to one's place and mask)
            let mut high = ((accumulator_data >> 4) & 0x0F).wrapping_sub((bus_data >> 4) & 0x0F);
            p.carry = true;

            // If the one's digit is negative, we need to borrow.
            if low & 0x80 == 0x80 {
                low = low.wrapping_add(10);
                high = high.wrapping_sub(1);
            }
            // If the one's digit is greater than nine, it's not valid decimal.
            // Adding 6 and taking the low nibble will fix this, since it causes
            // A -> F to wrap around back to the range 0 -> 5.
            else if low > 9 {
                low = low.wrapping_add(6) & 0x0F;
            }

            // If the ten's digit is negative, we need to borrow from the carry bit.
            if high & 0x80 == 0x80 {
                high = high.wrapping_add(10);
                p.carry = false;
            }
            // Wrap high digit as well, same as low
            else if high > 9 {
                low = low.wrapping_add(6) & 0x0F;
            }

            // Rebuild the result
            sum = (high << 4) | low;

            p.zero = sum == 0;
            is_negative = sum & 0x80 == 0x80;
        } else {
            let bin = (accumulator_data as u16)
                .wrapping_add((!bus_data) as u16)
                .wrapping_add(p.carry as u16);

            // Carry = inverse of borrow
            p.carry = bin & 0x100 == 0x100;

            sum = (bin & 0xFF) as u8;
            p.zero = sum == 0;
            is_negative = sum & 0x80 == 0x80;
        }

        self.accumulator.borrow_mut().write(sum);
        p.negative = is_negative;
        p.overflow = was_negative ^ is_negative;
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
    /// External bus
    bus: Rc<RefCell<Bus>>,
    /// Number of cycles remaining in current instruction
    cycles: u32,
    /// Number of input clocks since startup.
    clocks: u32,
    #[allow(dead_code)]
    ready: bool,
    not_irq: bool,
    #[allow(dead_code)]
    not_nmi: bool,
    #[allow(dead_code)]
    not_set_overflow: bool,
    #[allow(dead_code)]
    sync: bool,
    #[allow(dead_code)]
    not_reset: bool,
    interrupt_vector: u16,
}

enum IndexRegister {
    X,
    Y,
}

impl Mos6502 {
    /// Initializes a new `Mos6502` processor emulator.
    pub fn new(bus: &Rc<RefCell<Bus>>) -> Self {
        let data_bus = DataBus::new();
        let data_bus = Rc::new(RefCell::new(data_bus));

        let address_bus = AddressBus::new();
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
            bus: Rc::clone(bus),
            cycles: 0,
            clocks: 0,
            ready: false,
            not_irq: true,
            not_nmi: true,
            not_reset: true,
            not_set_overflow: true,
            sync: false,
            interrupt_vector: 0xFFFE,
        }
    }

    fn write_address(&mut self, address_high: u8, address_low: u8) {
        self.address_bus
            .borrow_mut()
            .write(address_high, address_low);
    }

    fn read(&mut self) -> u8 {
        let address = self.address_bus.borrow().address();
        let data = self.bus.borrow_mut().cpu_read(address);
        self.data_bus.borrow_mut().write(data);
        self.data_bus.borrow().read()
    }

    fn write(&mut self) {
        self.bus.borrow_mut().cpu_write(
            self.address_bus.borrow().address(),
            self.data_bus.borrow().read(),
        );
    }

    /// Runs the processor for a single clock cycle.
    ///
    /// Really, it does everything in one go on the
    /// first clock cycle and then spends the rest of
    /// the time doing nothing.
    ///
    /// Returns true if the instruction is complete.
    pub fn clock(&mut self) -> bool {
        // Divide input clock by 12.
        if self.clocks % 12 != 0 {
            self.clocks = self.clocks.wrapping_add(1);
            return false;
        }

        if self.cycles == 0 {
            self.read_instruction();

            if !self.p.irq_disable && !self.not_irq {
                self.interrupt_vector = 0xFFFE;
                // Set instruction register to 0, the BRK instruction.
                self.instruction_register.data = 0x00;
            }

            if !self.not_nmi {
                self.interrupt_vector = 0xFFFB;
                // Set instruction register to 0, the BRK instruction.
                self.instruction_register.data = 0x00;
            }

            self.execute_instruction();
        }

        self.cycles -= 1;
        self.clocks = self.clocks.wrapping_add(1);

        self.cycles == 0
    }

    fn fetch_next_byte(&mut self) -> u8 {
        self.pc.borrow_mut().increment();
        self.pc.borrow().write_to_address_bus();
        self.read()
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
            .write(address_high, address_low);
        self.read();
    }

    fn do_addressing_mode(&mut self, mode: AddressingMode) {
        match mode {
            AddressingMode::Absolute => {
                let address_low = self.fetch_next_byte();
                let address_high = self.fetch_next_byte();
                self.address_bus
                    .borrow_mut()
                    .write(address_high, address_low);
                self.read();
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
            AddressingMode::Implied => return,
            AddressingMode::Indirect => {
                let zero_page_offset = self.fetch_next_byte();

                self.write_address(0, zero_page_offset);
                let address_low = self.read();

                self.write_address(0, zero_page_offset + 1);
                let address_high = self.read();

                self.write_address(address_high, address_low);
                self.read();
            }
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
                self.read();
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
                    address_high + 1
                } else {
                    address_high
                };
                self.write_address(address_high, address_low);
                self.read();
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
                self.write_address(0, zero_page_offset);
                self.read();
            }
            AddressingMode::ZeroPageX => {
                let zero_page_offset = self.fetch_next_byte();
                let zero_page_offset = zero_page_offset.wrapping_add(self.x);
                self.write_address(0, zero_page_offset);
                self.read();
            }
            AddressingMode::ZeroPageY => {
                let zero_page_offset = self.fetch_next_byte();
                let zero_page_offset = zero_page_offset.wrapping_add(self.y);
                self.write_address(0, zero_page_offset);
                self.read();
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

    fn compare(&mut self, mode: AddressingMode, operand: u8, cycles: u32) {
        self.cycles = cycles;

        self.do_addressing_mode(mode);
        let memory = self.data_bus.borrow().read();

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

        self.read();
        self.pc.borrow_mut().read_low_from_data_bus();

        self.fetch_next_byte();

        self.read();
        self.pc.borrow_mut().read_high_from_data_bus();
    }

    fn interrupt(&mut self, cycles: u32, bytes: u32) {
        self.cycles = cycles;

        let address = self.pc.borrow().wide();
        // Store the next instruction in the stack
        let address = address.wrapping_add(bytes as u16);
        let high = ((address >> 4) & 0x0F) as u8;
        let low = (address & 0x0F) as u8;

        self.write_address(0x01, self.s);
        self.data_bus.borrow_mut().write(high);
        self.write();

        self.s -= 1;

        self.write_address(0x01, self.s);
        self.data_bus.borrow_mut().write(low);
        self.write();

        self.s -= 1;

        self.write_address(0x01, self.s);
        self.data_bus.borrow_mut().write(self.p.get());
        self.write();

        let interrupt_vector = self.interrupt_vector;
        let vector_high = ((interrupt_vector & 0xFF00) >> 8) as u8;
        let vector_low = (interrupt_vector & 0xFF) as u8;

        self.write_address(vector_high, vector_low);
        self.read();
        self.pc.borrow_mut().read_low_from_data_bus();

        let interrupt_vector = interrupt_vector.wrapping_sub(1);
        let vector_high = ((interrupt_vector & 0xFF00) >> 8) as u8;
        let vector_low = (interrupt_vector & 0xFF) as u8;

        self.write_address(vector_high, vector_low);
        self.read();
        self.pc.borrow_mut().read_high_from_data_bus();
    }

    fn read_instruction(&mut self) {
        self.pc.borrow().write_to_address_bus();
        self.read();
        self.instruction_register.read_from_bus();
    }

    fn execute_instruction(&mut self) {
        let instruction = self.instruction_register.decode_instruction();
        match instruction {
            Instruction::ADC(mode, _, cycles) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.alu.add_with_carry(&mut self.p);
            }
            Instruction::AND(mode, _, cycles) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);

                let operand = self.data_bus.borrow().read();
                let result = operand & self.a.borrow().read();

                self.a.borrow_mut().write(result);

                self.p.zero = result == 0;
                self.p.negative = result & 0x80 == 0x80;
            }
            Instruction::ASL(mode, _, cycles) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                let operand = self.data_bus.borrow().read();
                let result = operand << 1;
                self.data_bus.borrow_mut().write(result);

                self.p.zero = result == 0;
                self.p.negative = result & 0x80 == 0x80;
                self.p.carry = operand & 0x80 == 0x80;

                if mode == AddressingMode::Accumulator {
                    self.a.borrow_mut().read_from_bus();
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
                let operand = self.data_bus.borrow().read();
                self.p.negative = operand & 0x80 == 0x80;
                self.p.overflow = operand & 0x40 == 0x40;
                self.p.zero = operand & self.a.borrow().read() == 0;
            }
            Instruction::BMI(mode, _, cycles) => self.branch(self.p.negative, mode, cycles),
            Instruction::BNE(mode, _, cycles) => self.branch(!self.p.zero, mode, cycles),
            Instruction::BPL(mode, _, cycles) => self.branch(!self.p.negative, mode, cycles),
            Instruction::BRK(_, bytes, cycles) => self.interrupt(cycles, bytes),
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
                let operand = self.a.borrow().read();
                self.compare(mode, operand, cycles)
            }
            Instruction::CPX(mode, _, cycles) => self.compare(mode, self.x, cycles),
            Instruction::CPY(mode, _, cycles) => self.compare(mode, self.y, cycles),
            Instruction::DEC(mode, _, cycles) => {
                self.do_addressing_mode(mode);
                let memory = self.data_bus.borrow().read();
                let result = self.increment(memory, NEGATIVE_ONE, cycles);

                self.data_bus.borrow_mut().write(result);
                self.write();
            }
            Instruction::DEX(_, _, cycles) => self.x = self.increment(self.x, NEGATIVE_ONE, cycles),
            Instruction::DEY(_, _, cycles) => self.y = self.increment(self.y, NEGATIVE_ONE, cycles),
            Instruction::EOR(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);
                let operand = self.data_bus.borrow().read();

                let result = self.a.borrow().read() ^ operand;
                self.a.borrow_mut().write(result);

                self.p.zero = result == 0;
                self.p.negative = result & 0x80 == 0x80;
            }
            Instruction::INC(mode, _, cycles) => {
                self.do_addressing_mode(mode);
                let operand = self.data_bus.borrow().read();

                let result = self.increment(operand, 1, cycles);
                self.data_bus.borrow_mut().write(result);
                self.write();
            }
            Instruction::INX(_, _, cycles) => self.x = self.increment(self.x, 1, cycles),
            Instruction::INY(_, _, cycles) => self.y = self.increment(self.y, 1, cycles),
            Instruction::JMP(mode, _, cycles) => self.jump(mode, cycles),
            Instruction::JSR(mode, bytes, cycles) => {
                let next_address = self
                    .pc
                    .borrow()
                    .wide()
                    .wrapping_add((bytes & 0xFFFF) as u16)
                    // We want the next address to be the last byte of this instruction,
                    // not the first byte of the next, so we subtract one.
                    .wrapping_sub(1);
                let next_address_high = ((next_address & 0xFF00) >> 8) as u8;
                let next_address_low = (next_address & 0x00FF) as u8;

                // Save next instruction location to the stack.
                self.write_address(0x01, self.s);
                self.data_bus.borrow_mut().write(next_address_high);
                self.write();
                self.s -= 1;

                self.write_address(0x01, self.s);
                self.data_bus.borrow_mut().write(next_address_low);
                self.write();
                self.s -= 1;

                self.jump(mode, cycles);
            }
            Instruction::LDA(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);
                self.a.borrow_mut().read_from_bus();
            }
            Instruction::LDX(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);
                self.x = self.data_bus.borrow().read();
            }
            Instruction::LDY(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);
                self.y = self.data_bus.borrow().read();
            }
            Instruction::LSR(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);
                let operand = self.data_bus.borrow().read();
                self.p.carry = operand & 0x01 == 0x01;

                let result = operand >> 1;
                self.data_bus.borrow_mut().write(result);
                self.p.zero = result == 0x00;
                self.p.negative = false;

                if mode == AddressingMode::Accumulator {
                    self.a.borrow_mut().read_from_bus();
                } else {
                    self.write();
                }
            }
            Instruction::NOP(_, _, cycles) => self.cycles = cycles,
            Instruction::ORA(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);

                let memory = self.data_bus.borrow_mut().read();
                let result = self.a.borrow().read() | memory;

                self.a.borrow_mut().write(result);
                self.p.zero = result == 0;
                self.p.negative = result & 0x80 == 0x80;
            }
            Instruction::PHA(_, _, cycles) => {
                self.cycles = cycles;

                self.write_address(0x01, self.s);
                self.data_bus.borrow_mut().write(self.a.borrow().read());
                self.write();
                self.s -= 1;
            }
            Instruction::PHP(_, _, cycles) => {
                self.cycles = cycles;

                self.write_address(0x01, self.s);
                self.data_bus.borrow_mut().write(self.p.get());
                self.write();
                self.s -= 1;
            }
            Instruction::PLA(_, _, cycles) => {
                self.cycles = cycles;

                self.s += 1;
                self.write_address(0x01, self.s);
                let value = self.read();
                self.a.borrow_mut().write(value);
                self.p.negative = value & 0x80 == 0x80;
                self.p.zero = value == 0;
            }
            Instruction::PLP(_, _, cycles) => {
                self.cycles = cycles;

                self.s += 1;
                self.write_address(0x01, self.s);
                let value = self.read();
                self.p.set(value);
            }
            Instruction::ROL(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);
                let operand = self.data_bus.borrow().read();
                // Shift left and make bit 0 the carry bit
                let result = operand << 1 | (self.p.carry as u8);
                // The new carry bit is the old bit 7.
                self.p.carry = ((operand & 0x80) >> 7) != 0;
                self.p.negative = result & 0x80 == 0x80;
                self.p.zero = result == 0;

                self.data_bus.borrow_mut().write(result);

                if mode == AddressingMode::Accumulator {
                    self.a.borrow_mut().read_from_bus();
                } else {
                    self.write();
                }
            }
            Instruction::ROR(mode, _, cycles) => {
                self.cycles = cycles;

                self.do_addressing_mode(mode);
                let operand = self.data_bus.borrow().read();
                // Shift right and make bit 7 the carry bit
                let result = operand >> 1 | ((self.p.carry as u8) << 7);
                // The new carry is the old bit 0.
                self.p.carry = operand & 0x01 != 0;
                self.p.negative = result & 0x80 == 0x80;
                self.p.zero = result == 0;

                self.data_bus.borrow_mut().write(result);
                self.write();
            }
            Instruction::RTI(_, _, cycles) => {
                self.cycles = cycles;

                self.s += 1;
                self.write_address(0x01, self.s);
                let data = self.read();
                self.p.set(data);

                self.s += 1;
                self.write_address(0x01, self.s);
                self.read();
                self.pc.borrow_mut().read_low_from_data_bus();

                self.s += 1;
                self.write_address(0x01, self.s);
                self.read();
                self.pc.borrow_mut().read_high_from_data_bus();
            }
            Instruction::RTS(_, _, cycles) => {
                self.cycles = cycles;

                self.s += 1;
                self.write_address(0x01, self.s);
                self.read();
                self.pc.borrow_mut().read_low_from_data_bus();

                self.s += 1;
                self.write_address(0x01, self.s);
                self.read();
                self.pc.borrow_mut().read_high_from_data_bus();
            }
            Instruction::SBC(mode, _, cycles) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.alu.subtract_with_borrow(&mut self.p);
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
                self.a.borrow().write_to_bus();
                self.write();
            }
            Instruction::STX(mode, _, cycles) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.data_bus.borrow_mut().write(self.x);
                self.write();
            }
            Instruction::STY(mode, _, cycles) => {
                self.cycles = cycles;
                self.do_addressing_mode(mode);
                self.data_bus.borrow_mut().write(self.y);
                self.write();
            }
            Instruction::TAX(_, _, cycles) => {
                self.cycles = cycles;

                self.x = self.a.borrow().read();
                self.p.negative = self.x & 0x80 == 0x80;
                self.p.zero = self.x == 0;
            }
            Instruction::TAY(_, _, cycles) => {
                self.cycles = cycles;

                self.y = self.a.borrow().read();
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

                self.a.borrow_mut().write(self.x);
                self.p.negative = self.a.borrow().read() & 0x80 == 0x80;
                self.p.zero = self.a.borrow().read() == 0x00;
            }
            Instruction::TXS(_, _, cycles) => {
                self.cycles = cycles;

                self.s = self.x;
            }
            Instruction::TYA(_, _, cycles) => {
                self.cycles = cycles;

                self.a.borrow_mut().write(self.y);
                self.p.negative = self.a.borrow().read() & 0x80 == 0x80;
                self.p.zero = self.a.borrow().read() == 0x00;
            }
            Instruction::KIL => panic!(
                "{:x} instruction not implemented",
                self.instruction_register.data
            ),
        }

        self.pc.borrow_mut().increment();
    }
}

#[cfg(test)]
mod tests {
    use crate::assembler::{self, AssemblerError};
    use crate::cpu_bus::Bus;
    use std::rc::Rc;

    fn run_program(program: &str) -> Bus {
        match assembler::run_program(program) {
            Ok(connect) => match Rc::try_unwrap(connect) {
                Ok(connect) => connect.into_inner(),
                Err(_) => panic!("there is more than one strong reference"),
            },
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
        let mut bus = run_program(
            "
        LDA #$01
        ADC #$01
        STA $FF
        PHP
        ",
        );
        assert_eq!(bus.read(0xFF), 2, "0x1 + 0x1 = 0x2");
        assert_eq!(bus.read(0x01FD) & 0x01, 0x00, "carry bit cleared");

        let mut bus = run_program(
            "
            LDA #$FF
            ADC #$FF
            STA $FF
            PHP
        ",
        );
        assert_eq!(bus.read(0xFF), 0xFE, "0xFF + 0xFF = 0xFE");
        assert_eq!(bus.read(0x01FD) & 0x01, 0x01, "carry bit set");
    }

    #[test]
    fn adc_bcd() {
        let mut bus = run_program(
            "
            SED
            LDA #$10
            ADC #$10
            STA $FF
            PHP
        ",
        );
        assert_eq!(bus.read(0xFF), 0x20, "0x10 + 0x10 = 0x20 in BCD");
        assert_eq!(bus.read(0x01FD) & 0x01, 0x00, "carry bit cleared");

        let mut bus = run_program(
            "
            SED
            LDA #$81
            ADC #$92
            STA $FF
            PHP
            ",
        );
        assert_eq!(bus.read(0xFF), 0x73, "0x81 + 0x92 = 0x73 in BCD");
        assert_eq!(bus.read(0x01FD) & 0x01, 0x01, "0x81 + 0x92 sets carry flag");
    }

    #[test]
    fn and() {
        let mut bus = run_program(
            "
            LDA #$FF
            STA $FF
            LDA #$AA
            AND #$55
            STA $FF
            PHP
        ",
        );
        assert_eq!(bus.read(0xFF), 0x00, "(0xAA & 0x55) = 0x00");
        let status = bus.read(0x01FD);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag cleared");

        let mut bus = run_program(
            "
            LDA #$FF
            AND #$80
            STA $FF
            PHP
        ",
        );
        assert_eq!(bus.read(0xFF), 0x80, "0xFF & 0x80 = 0x80");
        assert_eq!(bus.read(0x01FD) & 0x80, 0x80, "negative bit set");
        assert_eq!(bus.read(0x01FD) & 0x02, 0x00, "zero bit not set");
    }

    #[test]
    fn asl() {
        let mut bus = run_program(
            "
        LDA #$FF
        ASL
        STA $FF
        PHP
        ",
        );
        let status = bus.read(0x01FD);
        assert_eq!(bus.read(0xFF), 0xFE, "asl result correct");
        assert!(status & 0x80 == 0x80, "negative bit set");
        assert!(status & 0x02 == 0x00, "zero bit not set");
        assert!(status & 0x01 == 0x01, "carry bit set");
    }

    #[test]
    fn bcc() {
        let mut bus = run_program(
            "
        LDA #$FE
        ADC #$03 // Result is 0x101, carry set
        BCC $02  // Branch should not be taken, next line executes
        LDA #$FF
        STA $FF  // 0xFF stored to $FF
        ",
        );

        assert_eq!(bus.read(0xFF), 0xFF, "branch not taken");

        let mut bus = run_program(
            "
        LDA #$FE
        ADC #$01 // Result is 0xFF, carry cleared
        BCC $02  // Branch should be taken to STA $FF
        LDA #$FA
        STA $FF  // 0xFF stored to $FF
        ",
        );

        assert_eq!(bus.read(0xFF), 0xFF, "branch taken");
    }

    #[test]
    fn bcs() {
        let mut bus = run_program(
            "
        LDA #$FE
        ADC #$03 // Result is 0x101, carry set
        BCS $02  // Branch should be taken to STA $FF
        LDA #$FF
        STA $FF  // 0x01 stored to $FF
        ",
        );

        assert_eq!(bus.read(0xFF), 0x01, "branch taken");

        let mut bus = run_program(
            "
        LDA #$FE
        ADC #$01 // Result is 0xFF, carry cleared
        BCS $02  // Branch should not be taken, next line executes
        LDA #$FA
        STA $FF  // 0xFA stored to $FF
        ",
        );

        assert_eq!(bus.read(0xFF), 0xFA, "branch not taken");
    }

    #[test]
    fn beq() {
        let mut bus = run_program(
            "
        SEC
        LDA #$FF
        SBC #$FF // Result is 0x00, zero set
        BEQ $02  // Branch should be taken to STA $FF
        LDA #$FF
        STA $FF  // 0x00 stored to $FF
        ",
        );

        assert_eq!(bus.read(0xFF), 0x00, "branch taken");
        let mut bus = run_program(
            "
        SEC
        LDA #$FF
        SBC #$FE // Result is 0x01, zero cleared
        BEQ $02  // Result should not be taken, next line executes
        LDA #$FF
        STA $FF  // 0xFF stored to $FF
        ",
        );

        assert_eq!(bus.read(0xFF), 0xFF, "branch not taken");
    }

    #[test]
    fn bit() {
        let mut bus = run_program(
            "
        LDA #$AA
        STA $FF
        LDA #$55
        BIT $FF
        PHP
        ",
        );

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(status & 0x40, 0x00, "overflow flag unset");
        assert_eq!(status & 0x02, 0x02, "zero flag set");
    }

    #[test]
    fn bmi() {
        let mut bus = run_program(
            "
        SEC
        LDA #$00
        SBC #$01 // Result is 0xFF, negative bit set
        BMI $02  // Branch should be taken to STA $FF
        LDA #$01
        STA $FF  // 0xFF stored to $FF 
        ",
        );

        assert_eq!(bus.read(0xFF), 0xFF, "branch taken");

        let mut bus = run_program(
            "
        SEC
        LDA #$01
        SBC #$01 // Result is 0x00, negative bit not set
        BMI $02  // Branch should not be taken, next line executes
        LDA #$02
        STA $FF  // 0x02 stored to $FF
        ",
        );

        assert_eq!(bus.read(0xFF), 0x02, "branch not taken");
    }

    #[test]
    fn bne() {
        let mut bus = run_program(
            "
        SEC
        LDA #$FF
        SBC #$FF // Result is 0x00, zero set
        BNE $02  // Branch should not be taken, next line executes
        LDA #$FF
        STA $FF  // 0xFF stored to $FF
        ",
        );

        assert_eq!(bus.read(0xFF), 0xFF, "branch not taken");

        let mut bus = run_program(
            "
        SEC
        LDA #$FF
        SBC #$FE // Result is 0x01, zero cleared
        BNE $02  // Branch should be taken to STA $FF
        LDA #$FF
        STA $FF  // 0x01 stored to $FF
        ",
        );

        assert_eq!(bus.read(0xFF), 0x01, "branch taken");
    }

    #[test]
    fn bpl() {
        let mut bus = run_program(
            "
        SEC
        LDA #$00
        SBC #$01 // Result is 0xFF, negative bit set
        BPL $02  // Branch should not be taken to STA $FF
        LDA #$01
        STA $FF  // 0xFF stored to $FF 
        ",
        );

        assert_eq!(bus.read(0xFF), 0x01, "branch taken");

        let mut bus = run_program(
            "
        SEC
        LDA #$04
        SBC #$01 // Result is 0x00, negative bit not set
        BPL $02  // Branch should be taken, next line executes
        LDA #$02
        STA $FF  // 0x03 stored to $FF
        ",
        );

        assert_eq!(bus.read(0xFF), 0x03, "branch not taken");
    }

    #[test]
    fn brk() {
        let mut bus = run_program(
            "
            SEC
            LDA #$AA
            SBC #$AA
            BRK
        ",
        );

        assert_eq!(bus.read(0x01FD), 0x00, "address after BRK stored on stack");
        assert_eq!(bus.read(0x01FC), 0x07, "address after BRK stored on stack");
        assert_eq!(bus.read(0x01FB) & 0x02, 0x02, "zero flag stored on stack");
        assert_eq!(bus.read(0x01FB) & 0x01, 0x01, "carry flag stored on stack");
    }

    #[test]
    fn bvc() {
        let mut bus = run_program(
            "
            LDA #$FF
            ADC #$05 // Result is 0x04, overflow set
            BVC $02  // Branch should not be taken, execute next instruction
            LDA #$FF
            STA $FF  // Store 0xFF in $FF 
        ",
        );

        assert_eq!(bus.read(0xFF), 0xFF, "branch not taken");

        let mut bus = run_program(
            "
            LDA #$01
            ADC #$05 // Result is 0x06, overflow not set
            BVC $02  // Branch should be taken, continue with STA $FF
            LDA #$FF
            STA $FF  // Store 0x06 in $FF 
        ",
        );

        assert_eq!(bus.read(0xFF), 0x06, "branch taken");
    }

    #[test]
    fn bvs() {
        let mut bus = run_program(
            "
            LDA #$FF
            ADC #$05 // Result is 0x04, overflow set
            BVS $02  // Branch should be taken, continue with STA $FF
            LDA #$FF
            STA $FF  // Store 0x04 in $FF 
        ",
        );

        assert_eq!(bus.read(0xFF), 0x04, "branch taken");

        let mut bus = run_program(
            "
            LDA #$01
            ADC #$05 // Result is 0x06, overflow not set
            BVS $02  // Branch should not be taken, continue with STA $FF
            LDA #$FF
            STA $FF  // Store 0xFF in $FF 
        ",
        );

        assert_eq!(bus.read(0xFF), 0xFF, "branch taken");
    }

    #[test]
    fn cmp() {
        let mut bus = run_program(
            "
            LDA #$10
            CMP #$05
            PHP
            ",
        );

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");

        let mut bus = run_program(
            "
            LDA #$10
            CMP #$10
            PHP
            ",
        );

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x02, "zero flag set");

        let mut bus = run_program(
            "
            LDA #$10
            CMP #$11
            PHP
            ",
        );

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");
    }

    #[test]
    fn cpx() {
        let mut bus = run_program(
            "
            LDX #$10
            CPX #$05
            PHP
            ",
        );

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");

        let mut bus = run_program(
            "
            LDX #$10
            CPX #$10
            PHP
            ",
        );

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x02, "zero flag set");

        let mut bus = run_program(
            "
            LDX #$10
            CPX #$11
            PHP
            ",
        );

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");
    }

    #[test]
    fn cpy() {
        let mut bus = run_program(
            "
            LDY #$10
            CPY #$05
            PHP
            ",
        );

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");

        let mut bus = run_program(
            "
            LDY #$10
            CPY #$10
            PHP
            ",
        );

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative flag not set");
        assert_eq!(status & 0x02, 0x02, "zero flag set");

        let mut bus = run_program(
            "
            LDY #$10
            CPY #$11
            PHP
            ",
        );

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(status & 0x02, 0x00, "zero flag not set");
    }

    #[test]
    fn dec() {
        let mut bus = run_program(
            "
            LDA #$02
            STA $FF
            LDA #$01
            STA $FF
            DEC $FF
            PHP
        ",
        );

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(bus.read(0xFF), 0x00, "correct result");

        let mut bus = run_program(
            "
            LDA #$02
            STA $FF
            LDA #$00
            STA $FF
            DEC $FF
            PHP
        ",
        );

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(bus.read(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn dex() {
        let mut bus = run_program(
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

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(bus.read(0xFF), 0x00, "correct result");

        let mut bus = run_program(
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

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(bus.read(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn dey() {
        let mut bus = run_program(
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

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(bus.read(0xFF), 0x00, "correct result");

        let mut bus = run_program(
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

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(bus.read(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn eor() {
        let mut bus = run_program(
            "
            LDA #$55
            EOR #$AA
            STA $FF
            PHP
        ",
        );

        assert_eq!(bus.read(0xFF), 0xFF, "0x55 xor 0xAA = 0xFF");
        assert_eq!(bus.read(0x01FD) & 0x80, 0x80, "negative bit set");

        let mut bus = run_program(
            "
            LDA #$FF
            EOR #$FF
            STA $FF
            PHP
        ",
        );

        assert_eq!(bus.read(0xFF), 0x00, "0xFF xor 0xFF = 0x00");
        assert_eq!(bus.read(0x01FD) & 0x02, 0x02, "zero bit set");
    }

    #[test]
    fn inc() {
        let mut bus = run_program(
            "
            LDA #$02
            STA $FF
            LDA #$FF
            STA $FF
            INC $FF
            PHP
        ",
        );

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(bus.read(0xFF), 0x00, "correct result");

        let mut bus = run_program(
            "
            LDA #$02
            STA $FF
            LDA #$FE
            STA $FF
            INC $FF
            PHP
        ",
        );

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(bus.read(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn inx() {
        let mut bus = run_program(
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

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(bus.read(0xFF), 0x00, "correct result");

        let mut bus = run_program(
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

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(bus.read(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn iny() {
        let mut bus = run_program(
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

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x02, 0x02, "zero flag set");
        assert_eq!(status & 0x80, 0x00, "negative flag unset");
        assert_eq!(bus.read(0xFF), 0x00, "correct result");

        let mut bus = run_program(
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

        let status = bus.read(0x01FD);
        assert_eq!(status & 0x02, 0x00, "zero flag unset");
        assert_eq!(status & 0x80, 0x80, "negative flag set");
        assert_eq!(bus.read(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn jmp() {
        let mut bus = run_program(
            "
            JMP $0800
            NOP
            NOP
            LDA #$FF // These two lines should not execute
            STA $FF  // so $FF should be empty.
            NOP
        ",
        );

        assert_eq!(bus.read(0xFF), 0x00, "load and store jumped over");
    }

    #[test]
    fn jsr() {
        let mut bus = run_program(
            "
            JSR $0800
            LDA #$FF
            STA $FF
            LDA #$FF
            STA $FE
        ",
        );

        assert_ne!(bus.read(0xFF), 0xFF, "first store skipped");
        assert_ne!(bus.read(0x00FE), 0xFF, "second store skipped");
        assert_eq!(bus.read(0x01FD), 0x00, "high byte = 0x00");
        assert_eq!(bus.read(0x01FC), 0x02, "low byte = 0x02");
    }

    #[test]
    fn lsr() {
        let mut bus = run_program(
            "
        LDA #$FF
        STA $FF
        LSR $FF
        PHP
       ",
        );

        assert_eq!(bus.read(0xFF), 0x7F, "0xFF >> 1 = 0x7F");
        let status = bus.read(0x01FD);
        assert_eq!(status & 0x01, 0x01, "carry bit set");
        assert_eq!(status & 0x02, 0x00, "zero bit unset");
        assert_eq!(status & 0x80, 0x00, "negative bit unset");

        let mut bus = run_program(
            "
            LDA #$01
            STA $FF
            LSR $FF
            PHP
        ",
        );

        assert_eq!(bus.read(0xFF), 0x00, "0x01 >> 1 = 0x00");
        let status = bus.read(0x01FD);
        assert_eq!(status & 0x01, 0x01, "carry bit set");
        assert_eq!(status & 0x02, 0x02, "zero bit set");
        assert_eq!(status & 0x80, 0x00, "negative bit unset");
    }

    #[test]
    fn ora() {
        let mut bus = run_program(
            "
            LDA #$AA
            STA $FF
            LDA #$55
            ORA $FF
            STA $FF
            PHP
        ",
        );

        assert_eq!(bus.read(0xFF), 0xFF, "0xAA | 0x55 = 0xFF");
        let status = bus.read(0x01FD);
        assert_eq!(status & 0x80, 0x80, "result negative");
        assert_eq!(status & 0x02, 0x00, "result not zero");

        let mut bus = run_program(
            "
            LDA #$00
            STA $FF
            LDA #$00
            ORA $FF
            STA $FF
            PHP
        ",
        );
        assert_eq!(bus.read(0xFF), 0x00, "0x00 | 0x00 = 0x00");
        let status = bus.read(0x01FD);
        assert_eq!(status & 0x80, 0x00, "result not negative");
        assert_eq!(status & 0x02, 0x02, "result zero");
    }

    #[test]
    fn pha() {
        let mut bus = run_program(
            "
            LDA #$FF
            PHA
        ",
        );

        assert_eq!(bus.read(0x01FD), 0xFF, "accumulator pushed on stack");
    }

    #[test]
    fn pla() {
        let mut bus = run_program(
            "
        LDA #$FF
        PHA
        LDA #$00
        PLA
        STA $FF
        ",
        );

        assert_eq!(bus.read(0x01FD), 0xFF, "accumulator pulled from stack");
    }

    #[test]
    fn rol() {
        let mut bus = run_program(
            "
        LDA #$FF
        STA $FF
        ROL $FF
        PHP
        ",
        );

        assert_eq!(bus.read(0x01FD) & 0x01, 0x01, "carry bit set");
        assert_eq!(bus.read(0xFF), 0xFE, "correct result");

        let mut bus = run_program(
            "
        LDA #$FF
        STA $FF
        SEC
        ROL $FF
        PHP
        ",
        );

        assert_eq!(bus.read(0x01FD) & 0x01, 0x01, "carry bit set");
        assert_eq!(bus.read(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn ror() {
        let mut bus = run_program(
            "
            LDA #$FF
            STA $FF
            ROR $FF
            PHP
        ",
        );

        assert_eq!(bus.read(0x01FD) & 0x01, 0x01, "carry bit set");
        assert_eq!(bus.read(0xFF), 0x7F, "correct result");

        let mut bus = run_program(
            "
            LDA #$FF
            STA $FF
            SEC
            ROR $FF
            PHP
        ",
        );

        assert_eq!(bus.read(0x01FD) & 0x01, 0x01, "carry bit set");
        assert_eq!(bus.read(0xFF), 0xFF, "correct result");
    }

    #[test]
    fn tax() {
        let mut bus = run_program(
            "
            LDA #$FF
            TAX
            STX $FF
            PHP
        ",
        );

        assert_eq!(bus.read(0xFF), 0xFF, "correct result");
        assert_eq!(bus.read(0x01FD) & 0x80, 0x80, "negative bit set");
    }

    #[test]
    fn sbc() {
        let mut bus = run_program(
            "
            LDA #$76
            SEC
            SBC #$05
            STA $FF
            PHP
        ",
        );
        assert_eq!(bus.read(0xFF), 0x71, "0x76 - 0x05 = 0x71");
        let status = bus.read(0x01FD);
        assert_eq!(status & 0x01, 0x01, "no borrow (carry set)");
        assert_eq!(status & 0x80, 0x00, "negative bit not set");
        assert_eq!(status & 0x02, 0x00, "zero bit not set");

        let mut bus = run_program(
            "
            ADC #$05
            SEC
            SBC #$0A
            STA $FF
            PHP
        ",
        );
        assert_eq!(bus.read(0xFF), 0xFB, "0x5 - 0xA = -0x5 (0xFB)");
        let status = bus.read(0x01FD);
        assert_eq!(status & 0x01, 0x00, "borrow (carry not set)");
        assert_eq!(status & 0x80, 0x80, "negative bit set");
        assert_eq!(status & 0x02, 0x00, "zero bit not set");
    }

    #[test]
    fn sbc_bcd() {
        let mut bus = run_program(
            "
            SED
            LDA #$92
            SEC
            SBC #$25
            STA $FF
            PHP
        ",
        );
        assert_eq!(bus.read(0xFF), 0x67);
        let status = bus.read(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative bit not set");
        assert_eq!(status & 0x02, 0x00, "zero bit not set");
        assert_eq!(status & 0x01, 0x01, "carry bit set");
        let mut bus = run_program(
            "
            SED
            LDA #$25
            SEC
            SBC #$92
            STA $FF
            PHP
        ",
        );
        assert_eq!(bus.read(0xFF), 0x33);
        let status = bus.read(0x01FD);
        assert_eq!(status & 0x80, 0x00, "negative bit not set");
        assert_eq!(status & 0x02, 0x00, "zero bit not set");
        assert_eq!(status & 0x01, 0x00, "carry bit not set");
    }
}
