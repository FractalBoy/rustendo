use crate::cartridge::Cartridge;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    cartridge: Option<Rc<RefCell<Cartridge>>>,
}

impl Bus {
    pub fn new() -> Self {
        Bus { cartridge: None }
    }

    pub fn load_cartridge(&mut self, cartridge: &Rc<RefCell<Cartridge>>) {
        self.cartridge = Some(Rc::clone(cartridge));
    }

    pub fn ppu_read(&self, address: u16) -> u8 {
        unimplemented!()
    }

    pub fn ppu_write(&mut self, address: u16, data: u8) {
        unimplemented!()
    }
}
