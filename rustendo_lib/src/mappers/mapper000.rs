use crate::cartridge::{Cartridge, Mapper};

pub struct Mapper000 {
    cartridge: Cartridge,
}

impl Mapper000 {
    pub fn new(cartridge: Cartridge) -> Self {
        Mapper000 { cartridge }
    }
}

impl Mapper for Mapper000 {
    fn read(&self, address: u16) -> u8 {
        unimplemented!()
    }

    fn write(&mut self, address: u16, data: u8) {
        unimplemented!()
    }
}
