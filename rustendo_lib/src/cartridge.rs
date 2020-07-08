use crate::mappers::mapper_000::Mapper000;
use crate::mappers::mapper_001::Mapper001;
use crate::mappers::Mapper;

#[derive(Debug)]
pub enum MirroringType {
    Vertical,
    Horizontal,
    OneScreen
}

#[derive(Debug)]
pub enum ConsoleType {
    NES,
    NintendoVsSystem,
    NintendoPlaychoice10,
    ExtendedConsoleType,
}

#[derive(Debug)]
pub enum TimingMode {
    NtscNes,
    PalNes,
    MultipleRegion,
    Dendy,
}

#[derive(Debug, PartialEq)]
pub enum CartridgeFormat {
    INes,
    Nes2,
}

pub struct Cartridge {
    raw: Vec<u8>,
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn new(raw: Vec<u8>) -> Self {
        let header = Self::_header(&raw);
        let mapper = match Self::_mapper(&header) {
            0 => Box::new(Mapper000::new(
                Self::_prg_rom_size(&header),
                Self::_chr_ram_size(&header),
            )) as Box<dyn Mapper>,
            1 => Box::new(Mapper001::new(Self::_chr_ram_size(&header))) as Box<dyn Mapper>,
            _ => unimplemented!(),
        };

        Cartridge { raw, mapper }
    }

    pub fn header(&self) -> &[u8] {
        Self::_header(&self.raw)
    }

    fn _header<'a>(raw: &'a [u8]) -> &'a [u8] {
        &raw[0..0x10]
    }

    pub fn format(&self) -> CartridgeFormat {
        Self::_format(self.header())
    }

    fn _format(header: &[u8]) -> CartridgeFormat {
        match header[7] & 0x0C {
            0x08 => CartridgeFormat::Nes2,
            _ => CartridgeFormat::INes,
        }
    }

    fn rom_size(size: u16, units: u16) -> u16 {
        match size & 0xF00 {
            0xF00 => {
                let multiplier = size & 0x3;
                let exponent = (size & 0xFC) >> 2;
                2u16.pow(exponent as u32) * (multiplier * 2 + 1)
            }
            _ => size * units,
        }
    }

    fn prg_rom_size(&self) -> usize {
        Self::_prg_rom_size(self.header())
    }

    fn _prg_rom_size(header: &[u8]) -> usize {
        match Self::_format(header) {
            CartridgeFormat::INes => ((header[4] as usize) * 0x4000),
            CartridgeFormat::Nes2 => {
                let lsb = header[4] as u16;
                let msb = ((header[9] as u16) & 0xF) << 4;
                let size = msb | lsb;

                Self::rom_size(size, 0x4000) as usize
            }
        }
    }

    pub fn prg_rom(&self) -> &[u8] {
        let start = 0x10 + self.trainer_size();
        let end = start + self.prg_rom_size();
        &self.raw[start..end]
    }

    pub fn chr_rom_size(&self) -> usize {
        Self::_chr_rom_size(self.header())
    }

    fn _chr_rom_size(header: &[u8]) -> usize {
        match Self::_format(header) {
            CartridgeFormat::INes => (header[5] as usize) * 0x2000,
            CartridgeFormat::Nes2 => {
                let lsb = header[5] as u16;
                let msb = (header[9] as u16) & 0xF0;
                let size = msb << 4 | lsb;

                Self::rom_size(size, 0x2000) as usize
            }
        }
    }

    pub fn chr_rom(&self) -> &[u8] {
        let start = 0x10 + self.trainer_size() + self.prg_rom_size();
        let end = start + self.chr_rom_size();
        &self.raw[start..end]
    }

    pub fn miscellaneous_rom(&self) -> &[u8] {
        let start = 0x10 + self.trainer_size() + self.prg_rom_size() + self.chr_rom_size();
        let end = self.raw.len();
        &self.raw[start..end]
    }

    pub fn prg_ram_size(&self) -> usize {
        match self.format() {
            CartridgeFormat::INes => match self.header()[8] {
                0 => 0x2000,
                size => size as usize,
            },
            CartridgeFormat::Nes2 => match self.header()[10] & 0x0F {
                0 => 0,
                shift_count => 64 << shift_count,
            },
        }
    }

    pub fn prg_nvram_size(&self) -> usize {
        let shift_count = self.header()[10] & 0xF0;
        match shift_count {
            0 => 0,
            _ => 64 << shift_count,
        }
    }

    fn _chr_ram_size(header: &[u8]) -> usize {
        match Self::_format(header) {
            CartridgeFormat::INes => match header[5] {
                0 => 0x1FE000,
                _ => 0,
            },
            CartridgeFormat::Nes2 => match header[11] & 0x0F {
                0 => 0,
                shift_count => 64 << shift_count,
            },
        }
    }

    pub fn chr_ram_size(&self) -> usize {
        Self::_chr_ram_size(self.header())
    }

    pub fn chr_nvram_size(&self) -> usize {
        let shift_count = self.header()[11] & 0xF0;
        match shift_count {
            0 => 0,
            _ => 64 << shift_count,
        }
    }

    pub fn has_trainer(&self) -> bool {
        self.header()[6] & 0x4 == 0x4
    }

    pub fn trainer(&self) -> &[u8] {
        let start: usize = 0x10;
        let end = 0x10 + self.trainer_size() as usize;
        &self.raw[start..end]
    }

    fn trainer_size(&self) -> usize {
        if self.has_trainer() {
            0x200
        } else {
            0
        }
    }

    pub fn mirroring_type(&self) -> MirroringType {
        match self.mapper.mirroring_type() {
            None => {
                if self.header()[6] & 0x1 == 0x1 {
                    MirroringType::Vertical
                } else {
                    MirroringType::Horizontal
                }
            },
            Some(mirroring) => mirroring
        }
    }

    pub fn has_battery(&self) -> bool {
        self.header()[6] & 0x2 == 0x2
    }

    pub fn hard_wired_four_screen_mode(&self) -> bool {
        self.header()[6] & 0x8 == 0x8
    }

    fn _mapper(header: &[u8]) -> u16 {
        let d0_3 = ((header[6] as u16) & 0xF0) >> 4;
        let d4_7 = (header[7] as u16) & 0xF0;
        let d8_11 = ((header[8] as u16) & 0x0F) << 8;

        match Self::_format(header) {
            CartridgeFormat::INes => d4_7 | d0_3,
            CartridgeFormat::Nes2 => d8_11 | d4_7 | d0_3,
        }
    }

    pub fn mapper(&self) -> u16 {
        Self::_mapper(self.header())
    }

    pub fn submapper(&self) -> u8 {
        self.header()[8] & 0xF0 >> 4
    }

    pub fn console_type(&self) -> ConsoleType {
        match self.header()[7] & 0x3 {
            0 => ConsoleType::NES,
            1 => ConsoleType::NintendoVsSystem,
            2 => ConsoleType::NintendoPlaychoice10,
            3 => ConsoleType::ExtendedConsoleType,
            _ => unreachable!(),
        }
    }

    pub fn timing_mode(&self) -> TimingMode {
        match self.header()[12] & 0x2 {
            0x0 => TimingMode::NtscNes,
            0x1 => TimingMode::PalNes,
            0x2 => TimingMode::MultipleRegion,
            0x3 => TimingMode::Dendy,
            _ => unreachable!(),
        }
    }

    pub fn cpu_read(&self, address: u16) -> u8 {
        match self.mapper.cpu_read(address) {
            (Some(address), _) => self.prg_rom()[address],
            (_, Some(data)) => data,
            _ => 0,
        }
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        self.mapper.cpu_write(address, data);
    }

    pub fn ppu_read(&self, address: u16) -> u8 {
        match self.mapper.ppu_read(address) {
            (Some(address), _) => self.chr_rom()[address],
            (_, Some(data)) => data,
            _ => 0,
        }
    }

    pub fn ppu_write(&mut self, address: u16, data: u8) {
        self.mapper.ppu_write(address, data);
    }
}

#[cfg(test)]
mod tests {
    use super::{Cartridge, CartridgeFormat};
    use std::fs;
    use std::path::Path;

    fn get_cartridge() -> Cartridge {
        let current_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let nes_test = current_dir.parent().unwrap().join("nestest.nes");
        let buffer = fs::read(nes_test).unwrap();

        Cartridge::new(buffer)
    }

    #[test]
    fn format() {
        let cartridge = get_cartridge();
        assert_eq!(cartridge.format(), CartridgeFormat::INes, "format is iNES");
    }

    #[test]
    fn size() {
        let cartridge = get_cartridge();
        assert_eq!(cartridge.prg_rom_size(), 16384);
    }

    #[test]
    fn mapper() {
        let cartridge = get_cartridge();
        assert_eq!(cartridge.mapper(), 0);
    }
}
