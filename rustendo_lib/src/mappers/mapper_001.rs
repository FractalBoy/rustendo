use crate::cartridge::MirroringType;
use super::Mapper;

enum ControlBits {
    Mirroring = 0b00011,
    PrgRomBankMode = 0b01100,
    ChrRomBankMode = 0b10000,
}

bitfield!(Control, ControlBits, u8);

pub struct Mapper001 {
    chr_ram: Vec<u8>,
    prg_ram: [u8; 0x1FFF],
    shift_register: u8,
    control: Control,
    low_prg_space: usize,
    high_prg_space: usize,
    low_chr_space: usize,
    high_chr_space: usize,
}

enum Bank {
    High,
    Low,
}

impl Mapper001 {
    pub fn new(chr_ram_size: usize) -> Self {
        Mapper001 {
            prg_ram: [0; 0x1FFF],
            chr_ram: vec![0; chr_ram_size],
            shift_register: 0x10,
            control: Control::new(),
            high_prg_space: 0,
            low_prg_space: 0,
            high_chr_space: 0,
            low_chr_space: 0,
        }
    }

    fn set_register(&mut self, address: u16, data: u8) {
        match (address & 0x6000) >> 13 {
            0x0 => *self.control = data,
            0x1 => match self.control.get_field(ControlBits::ChrRomBankMode) {
                0x0 => {
                    // The lower bit is unused in 8 KiB mode.
                    let bank = ((data & 0x1E) >> 1) as usize;
                    // Each bank is always 0x1000 bytes in size and there are two banks.
                    // Therefore, in 8 KiB mode, the low CHR bank always starts every 0x2000
                    // bytes and the high CHR bank starts 0x1000 bytes after that
                    self.low_chr_space = bank * 0x2000;
                    self.high_chr_space = bank * 0x2000 + 0x1000;
                }
                0x1 => self.low_chr_space = ((data & 0x1F) as usize) * 0x1000,
                _ => unreachable!(),
            },
            0x2 => match self.control.get_field(ControlBits::ChrRomBankMode) {
                0x0 => (), // CHR bank 1 is ignored in 8KiB mode
                0x1 => self.high_chr_space = ((data & 0x1F) as usize) * 0x1000,
                _ => unreachable!(),
            },
            0x3 => match self.control.get_field(ControlBits::PrgRomBankMode) {
                0x0 | 0x1 => {
                    // The lower bit is unused in 8 KiB mode
                    let bank = ((data & 0xE) >> 1) as usize;
                    // Each bank is always 0x4000 bytes in size and there are two banks.
                    // Therefore, in 32 KiB mode, the low CHR bank always starts every 0x8000
                    // bytes and the high CHR bank starts 0x4000 bytes after that
                    self.low_prg_space = bank * 0x8000;
                    self.high_prg_space = bank * 0x8000 + 0x4000;
                }
                0x2 => self.high_prg_space = ((data & 0xF) as usize) * 0x4000,
                0x3 => self.low_prg_space = ((data & 0xF) as usize) * 0x4000,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn load_shift_register(&mut self, data: u8) {
        let mut data = data;
        // Make room for the new bit
        self.shift_register >>= 1;
        // Set bit 7 to 0
        self.shift_register &= 0x7F;
        // Get bit zero
        data &= 0x01;
        // Shift bit zero to bit 7
        data <<= 7;
        self.shift_register |= data;
    }

    fn get_prg_address(&self, bank: Bank, address: u16) -> usize {
        match bank {
            Bank::Low => self.low_prg_space + ((address & 0x3FFF) as usize),
            Bank::High => self.high_prg_space + ((address & 0x3FFF) as usize),
        }
    }

    fn get_chr_address(&self, bank: Bank, address: u16) -> usize {
        match bank {
            Bank::Low => self.low_chr_space + ((address & 0x0FFF) as usize),
            Bank::High => self.high_chr_space + ((address & 0x0FFF) as usize),
        }
    }
}

impl Mapper for Mapper001 {
    fn cpu_read(&self, address: u16) -> (Option<usize>, Option<u8>) {
        match address {
            // Unused, but in the cartridge's address range
            0x4020..=0x5FFF => (None, None),
            0x6000..=0x7FFF => (None, Some(self.prg_ram[(address & 0x1FFF) as usize])),
            // First bank
            0x8000..=0xBFFF => (Some(self.get_prg_address(Bank::Low, address)), None),
            // Second bank
            0xC000..=0xFFFF => (Some(self.get_prg_address(Bank::High, address)), None),
            _ => (None, None),
        }
    }
    fn cpu_write(&mut self, address: u16, data: u8) -> Option<usize> {
        match address {
            // Unused, but in the cartridge's address range
            0x4020..=0x5FFF => None,
            0x6000..=0x7FFF => {
                self.prg_ram[(address & 0x1FFF) as usize] = data;
                None
            }
            0x8000..=0xFFFF => {
                match data & 0x80 {
                    0x00 => {
                        let data = (data & 0x01) << 7;

                        // On the fifth shift, bit 0 of the register will be 1
                        if self.shift_register & 0x01 == 0x01 {
                            // Bit 4 of the shift register is loaded just to set
                            // the underlying register.
                            self.load_shift_register(data);
                            self.set_register(address, self.shift_register);
                            // Reset the shift register
                            self.shift_register = 0x10;
                        } else {
                            self.load_shift_register(data);
                        }
                    }
                    0x80 => {
                        self.shift_register = 0x10;
                    }
                    _ => unreachable!(),
                };
                None
            }
            _ => None,
        }
    }

    fn ppu_read(&self, address: u16) -> (Option<usize>, Option<u8>) {
        match address {
            0x0000..=0x0FFF => (Some(self.get_chr_address(Bank::Low, address)), None),
            0x1000..=0x1FFF => (Some(self.get_chr_address(Bank::High, address)), None),
            _ => (None, None),
        }
    }

    fn ppu_write(&mut self, _address: u16, _data: u8) -> Option<usize> {
        None
    }

    fn mirroring_type(&self) -> Option<MirroringType> { 
        match self.control.get_field(ControlBits::Mirroring) {
            0x0 | 0x1 => Some(MirroringType::OneScreen),
            0x2 => Some(MirroringType::Vertical),
            0x3 => Some(MirroringType::Horizontal),
            _ => unreachable!()
        }
    }
}
