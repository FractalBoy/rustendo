use crate::cartridge::Cartridge;
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
            ram: Ram::new(),
        }
    }

    pub fn load_cartridge(&mut self, cartridge: &Rc<RefCell<Cartridge>>) {
        self.ram
            .set_mirroring_type(cartridge.borrow().mirroring_type());
        self.cartridge = Some(Rc::clone(cartridge));
    }

    pub fn ppu_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => match &self.cartridge {
                Some(cartridge) => cartridge.borrow().ppu_read(address),
                None => 0,
            },
            0x2000..=0x3EFF => self.ram.read(address),
            _ => 0,
        }
    }

    pub fn ppu_write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => match &self.cartridge {
                Some(cartridge) => cartridge.borrow_mut().ppu_write(address, data),
                None => ()
            }
            0x2000..=0x3EFF => self.ram.write(address, data),
            _ => ()
        }
    }
}
