use super::Mapper;

pub struct Mapper000 {
    prg_rom_size: usize,
    chr_ram: Vec<u8>,
    prg_ram: [u8; 0x1FFF],
}

impl Mapper000 {
    pub fn new(prg_rom_size: usize, chr_ram_size: usize) -> Self {
        Mapper000 {
            prg_rom_size,
            chr_ram: vec![0; chr_ram_size],
            prg_ram: [0; 0x1FFF],
        }
    }
}

impl Mapper for Mapper000 {
    fn cpu_read(&self, address: u16) -> (Option<u16>, Option<u8>) {
        match address {
            // Unused, but in the cartridge's address range
            0x4020..=0x5FFF => (None, None),
            0x6000..=0x7FFF => (None, Some(self.prg_ram[(address & 0x1FFF) as usize])),
            0x8000..=0xBFFF => (Some(address & 0x7FFF), None),
            0xC000..=0xFFFF => match self.prg_rom_size {
                // If the size is 16 KiB, mirror
                0..=0x3FFF => (Some(address & 0xBFFF), None),
                // If the size is 32 KiB, continue previous range
                _ => (Some(address & 0x7FFF), None),
            },
            _ => (None, None),
        }
    }

    fn cpu_write(&mut self, address: u16, data: u8) -> Option<u16> {
        match address {
            0x6000..=0x7FFF => {
                self.prg_ram[(address & 0x1FFF) as usize] = data;
                None
            }
            _ => None,
        }
    }

    fn ppu_read(&self, address: u16) -> (Option<u16>, Option<u8>) {
        match address {
            0x0000..=0x1FFF => match self.chr_ram.len() {
                0 => (Some(address), None),
                _ => (None, Some(self.chr_ram[address as usize])),
            },
            _ => (None, None),
        }
    }

    fn ppu_write(&mut self, address: u16, data: u8) -> Option<u16> {
        match self.chr_ram.len() {
            0 => None,
            _ => {
                self.chr_ram[address as usize] = data;
                None
            }
        }
    }
}
