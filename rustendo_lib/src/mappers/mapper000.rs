use crate::cartridge::{Cartridge, Mapper};

pub struct Mapper000 {
    cartridge: Cartridge,
    prg_ram: Vec<u8>,
    chr_ram: Vec<u8>,
    // Non-volatile RAM should really be non-volatile, but for now
    // it will just be volatile.
    prg_nv_ram: Vec<u8>,
    chr_nv_ram: Vec<u8>,
}

impl Mapper000 {
    pub fn new(cartridge: Cartridge) -> Self {
        let prg_ram = vec![0; cartridge.prg_ram_size()];
        let chr_ram = vec![0; cartridge.chr_ram_size()];
        let prg_nv_ram = vec![0; cartridge.prg_nvram_size()];
        let chr_nv_ram = vec![0; cartridge.chr_nvram_size()];

        Mapper000 {
            cartridge,
            prg_ram,
            chr_ram,
            prg_nv_ram,
            chr_nv_ram,
        }
    }
}

impl Mapper for Mapper000 {
    fn cpu_read(&self, address: u16) -> u8 {
        match address {
            0x4020..=0x7FFF => 0,
            0x8000..=0xBFFF => 0,
            _ => 0,
        }
    }

    fn cpu_write(&mut self, address: u16, data: u8) {
        unimplemented!()
    }

    fn ppu_read(&self, address: u16) -> u8 {
        unimplemented!()
    }

    fn ppu_write(&mut self, address: u16, data: u8) {
        unimplemented!()
    }
}
