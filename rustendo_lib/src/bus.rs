use crate::cartridge::Cartridge;
use crate::nes2c02::Nes2c02;
use crate::ram::Ram;

pub struct Bus {
    pub ram: Ram,
    pub ppu: Nes2c02,
    cartridge: Option<Cartridge>,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: Ram::new(),
            ppu: Nes2c02::new(),
            cartridge: None,
        }
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = Some(cartridge);
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0..=0x1FFF => self.ram.write(address, data),
            _ => unimplemented!(),
        };
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0..=0x1FFF => self.ram.read(address).unwrap(),
            0x4020..=0xFFFF => match &self.cartridge {
                Some(cartridge) => cartridge.read(address),
                None => 0,
            },
            _ => 0
        }
    }

    pub fn clock(&mut self) {
        self.ppu.clock();
    }
}
