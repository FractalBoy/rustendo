use super::Mapper;

pub struct Mapper001 {
    chr_ram: Vec<u8>,
    prg_ram: [u8; 0x1FFF],
}

impl Mapper001 {
    pub fn new(chr_ram_size: usize) -> Self {
        Mapper001 {
            prg_ram: [0; 0x1FFF],
            chr_ram: vec![0; chr_ram_size],
        }
    }
}

impl Mapper for Mapper001 {
    fn cpu_read(&self, address: u16) -> (Option<u16>, Option<u8>) {
        match address {
            // Unused, but in the cartridge's address range
            0x4020..=0x5FFF => (None, None),
            0x6000..=0x7FFF => (None, Some(self.prg_ram[(address & 0x1FFF) as usize])),
            // First bank (switchable)
            0x8000..=0xBFFF => (Some(address & 0x7FFF), None),
            // Second bank (switchable)
            0xC000..=0xFFFF => (Some(address & 0x7FFF), None),
            _ => (None, None),
        }
    }
    fn cpu_write(&mut self, address: u16, data: u8) -> Option<u16> {
        match address {
            // Unused, but in the cartridge's address range
            0x4020..=0x5FFF => None,
            0x6000..=0x7FFF => {
                self.prg_ram[(address & 0x1FFF) as usize] = data;
                None
            }
            // First bank (switchable)
            0x8000..=0xBFFF => Some(address & 0x7FFF),
            // Second bank (switchable)
            0xC000..=0xFFFF => Some(address & 0x7FFF),
            _ => None,
        }
    }
    fn ppu_read(&self, address: u16) -> (Option<u16>, Option<u8>) {
        match self.chr_ram.len() {
            0 => (None, Some(self.chr_ram[address as usize])),
            _ => (Some(address), None),
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
