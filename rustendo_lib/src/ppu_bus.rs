use crate::cartridge::{Cartridge, Mapper};
use crate::mappers::mapper_000::Mapper000;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    mapper: Option<Box<dyn Mapper>>,
}

impl Bus {
    pub fn new() -> Self {
        Bus { mapper: None }
    }

    pub fn load_cartridge(&mut self, cartridge: &Rc<RefCell<Cartridge>>) {
        // The mapper is used to access the cartridge
        self.mapper = Some(Box::new(match cartridge.borrow().mapper() {
            0 => Mapper000::new(cartridge),
            mapper => unimplemented!("Mapper {} is unimplemented", mapper),
        }));
    }

    pub fn ppu_read(&self, address: u16) -> u8 {
        unimplemented!()
    }

    pub fn ppu_write(&mut self, address: u16, data: u8) {
        unimplemented!()
    }
}
