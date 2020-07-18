use crate::cartridge::{Cartridge, MirroringType};
use crate::ppu_ram::Ram;
use crate::ricoh2c02::Ricoh2c02;

pub struct Bus {
    ram: Ram,
    pub ppu: Ricoh2c02
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: Ram::new(),
            ppu: Ricoh2c02::new()
        }
    }

    pub fn ppu_read(&self, cartridge: &Option<Cartridge>, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => match cartridge {
                Some(cartridge) => cartridge.ppu_read(address),
                None => 0,
            },
            0x2000..=0x3EFF => match cartridge {
                Some(cartridge) => self.ram.read(cartridge.mirroring_type(), address),
                None => self.ram.read(MirroringType::Vertical, address),
            },
            _ => 0,
        }
    }

    pub fn ppu_write(&mut self, cartridge: &mut Option<Cartridge>, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => match cartridge {
                Some(cartridge) => cartridge.ppu_write(address, data),
                None => (),
            },
            0x2000..=0x3EFF => match cartridge {
                Some(cartridge) => {
                    self.ram
                        .write(cartridge.mirroring_type(), address, data)
                }
                None => self.ram.write(MirroringType::Vertical, address, data),
            },
            _ => (),
        }
    }

    pub fn cpu_read(&mut self, cartridge: &Option<Cartridge>, address: u16) -> u8 {
        let mut ppu = std::mem::replace(&mut self.ppu, Ricoh2c02::new());
        let data = ppu.cpu_read(self, cartridge, address);
        self.ppu = ppu;
        data
    }

    pub fn cpu_write(&mut self, cartridge: &mut Option<Cartridge>, address: u16, data: u8) {
        let mut ppu = std::mem::replace(&mut self.ppu, Ricoh2c02::new());
        ppu.cpu_write(self, cartridge, address, data);
        self.ppu = ppu;
    }

    pub fn clock(&mut self, cartridge: &mut Option<Cartridge>, nmi_enable: &mut bool) -> bool {
        let mut ppu = std::mem::replace(&mut self.ppu, Ricoh2c02::new());
        let frame_complete = ppu.clock(self, cartridge, nmi_enable);
        self.ppu = ppu;
        frame_complete
    }
}
