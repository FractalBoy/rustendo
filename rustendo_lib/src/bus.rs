use crate::cartridge::{Cartridge, Mapper};
use crate::mappers::mapper000::Mapper000;
use crate::nes2c02::Nes2c02;
use crate::ram::Ram;

pub struct Bus {
    pub ram: Ram,
    pub ppu: Nes2c02,
    mapper: Option<Box<dyn Mapper>>,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: Ram::new(),
            ppu: Nes2c02::new(),
            mapper: None,
        }
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        // The mapper is used to access the cartridge
        self.mapper = Some(Box::new(match cartridge.mapper() {
            0 => Mapper000::new(cartridge),
            mapper => unimplemented!("Mapper {} is unimplemented", mapper),
        }));
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            0x0..=0x1FFF => self.ram.write(address, data),
            0x4020..=0xFFFF => match &mut self.mapper {
                Some(mapper) => mapper.cpu_write(address, data),
                None => return,
            },
            _ => unimplemented!(),
        };
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x0..=0x1FFF => self.ram.read(address),
            0x4020..=0xFFFF => match &self.mapper {
                Some(mapper) => mapper.cpu_read(address),
                None => 0,
            },
            _ => 0,
        }
    }
}
