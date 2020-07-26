use crate::cartridge::{Cartridge, MirroringType};
use crate::ppu_ram::Ram;

pub struct Bus {
    ram: Box<Ram>,
    cartridge: Option<Box<Cartridge>>,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: Box::new(Ram::new()),
            cartridge: None,
        }
    }

    pub fn load_cartridge(&mut self, cartridge: Option<Box<Cartridge>>) {
        self.cartridge = cartridge
    }

    pub fn unload_cartridge(&mut self) -> Option<Box<Cartridge>> {
        self.cartridge.take()
    }

    pub fn ppu_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => match &self.cartridge {
                Some(cartridge) => cartridge.ppu_read(address),
                None => 0,
            },
            0x2000..=0x3EFF => match &self.cartridge {
                Some(cartridge) => self.ram.read(cartridge.mirroring_type(), address),
                None => self.ram.read(MirroringType::Vertical, address),
            },
            _ => 0,
        }
    }

    pub fn ppu_write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => match &mut self.cartridge {
                Some(cartridge) => cartridge.ppu_write(address, data),
                None => (),
            },
            0x2000..=0x3EFF => match &self.cartridge {
                Some(cartridge) => self.ram.write(cartridge.mirroring_type(), address, data),
                None => self.ram.write(MirroringType::Vertical, address, data),
            },
            _ => (),
        }
    }
}
