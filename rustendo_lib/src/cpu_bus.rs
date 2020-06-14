use crate::cartridge::Cartridge;
use crate::cpu_ram::Ram;
use crate::ricoh2c02::Ricoh2c02;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    pub ram: Ram,
    pub ppu: Ricoh2c02,
    cartridge: Option<Rc<RefCell<Cartridge>>>,
    fake_ram: [u8; 0x10000],
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: Ram::new(),
            ppu: Ricoh2c02::new(),
            cartridge: None,
            fake_ram: [0; 0x10000],
        }
    }

    pub fn load_cartridge(&mut self, cartridge: &Rc<RefCell<Cartridge>>) {
        self.cartridge = Some(Rc::clone(cartridge));
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => self.ram.write(address, data),
            0x2000..=0x3FFF => self.ppu.cpu_write(address & 0x2007, data),
            0x4020..=0xFFFF => match &self.cartridge {
                Some(mapper) => mapper.borrow_mut().cpu_write(address, data),
                None => self.fake_ram[address as usize] = data,
            },
            _ => self.fake_ram[address as usize] = data,
        };
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x0..=0x1FFF => self.ram.read(address),
            0x2000..=0x3FFF => self.ppu.cpu_read(address & 0x2007),
            0x4020..=0xFFFF => match &self.cartridge {
                Some(cartridge) => cartridge.borrow().cpu_read(address),
                None => self.fake_ram[address as usize],
            },
            _ => self.fake_ram[address as usize],
        }
    }
}
