use crate::cartridge::Cartridge;
use crate::nes2c02::Nes2c02;
use crate::cpu_ram::Ram;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    pub ram: Ram,
    pub ppu: Nes2c02,
    cartridge: Option<Rc<RefCell<Cartridge>>>,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: Ram::new(),
            ppu: Nes2c02::new(),
            cartridge: None,
        }
    }

    pub fn load_cartridge(&mut self, cartridge: &Rc<RefCell<Cartridge>>) {
        self.cartridge = Some(Rc::clone(cartridge));
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            0x0..=0x1FFF => self.ram.write(address, data),
            0x4020..=0xFFFF => match &self.cartridge {
                Some(mapper) => mapper.borrow_mut().cpu_write(address, data),
                None => return,
            },
            _ => unimplemented!(),
        };
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x0..=0x1FFF => self.ram.read(address),
            0x4020..=0xFFFF => match &self.cartridge {
                Some(cartridge) => cartridge.borrow().cpu_read(address),
                None => 0,
            },
            _ => 0,
        }
    }
}
