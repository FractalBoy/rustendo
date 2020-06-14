use crate::cartridge::Cartridge;
use crate::cpu_ram::Ram;
use crate::ricoh2c02::Ricoh2c02;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    pub ram: Ram,
    pub ppu: Ricoh2c02,
    cartridge: Option<Rc<RefCell<Cartridge>>>,
    // This ram is used only for testing.
    test_ram: Option<[u8; 0x10000]>,
}

impl Bus {
    pub fn new() -> Self {
        let mut bus = Bus {
            ram: Ram::new(),
            ppu: Ricoh2c02::new(),
            cartridge: None,
            test_ram: None,
        };

        // Set up some fake RAM to use for testing purposes only.
        if cfg!(test) {
            bus.test_ram = Some([0; 0x10000]);
        }

        bus
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
                None => self.set_test_ram(address, data),
            },
            _ => self.set_test_ram(address, data),
        };
    }

    fn set_test_ram(&mut self, address: u16, data: u8) {
        match &mut self.test_ram {
            Some(fake_ram) => fake_ram[address as usize] = data,
            None => return,
        }
    }

    fn get_test_ram(&self, address: u16) -> u8 {
        match &self.test_ram {
            Some(fake_ram) => fake_ram[address as usize],
            None => 0,
        }
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x0..=0x1FFF => self.ram.read(address),
            0x2000..=0x3FFF => self.ppu.cpu_read(address & 0x2007),
            0x4020..=0xFFFF => match &self.cartridge {
                Some(cartridge) => cartridge.borrow().cpu_read(address),
                None => self.get_test_ram(address),
            },
            _ => self.get_test_ram(address),
        }
    }
}
