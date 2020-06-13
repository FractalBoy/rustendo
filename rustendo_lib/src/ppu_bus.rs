use crate::cartridge::{Cartridge, MirroringType};
use crate::ppu_ram::Ram;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    cartridge: Option<Rc<RefCell<Cartridge>>>,
    ram: Ram,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            cartridge: None,
            ram: Ram::new(MirroringType::Vertical),
        }
    }

    pub fn load_cartridge(&mut self, cartridge: &Rc<RefCell<Cartridge>>) {
        self.ram = Ram::new(cartridge.borrow().mirroring_type());
        self.cartridge = Some(Rc::clone(cartridge));
    }

    pub fn ppu_read(&self, address: u16) -> u8 {
        unimplemented!()
    }

    pub fn ppu_write(&mut self, address: u16, data: u8) {
        unimplemented!()
    }
}
