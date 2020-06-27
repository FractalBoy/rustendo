use crate::cartridge::Cartridge;
use crate::controller::Controller;
use crate::cpu_ram::Ram;
use crate::ricoh2c02::Ricoh2c02;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    ram: Ram,
    ppu: Rc<RefCell<Ricoh2c02>>,
    cartridge: Option<Rc<RefCell<Cartridge>>>,
    controller: Rc<RefCell<Controller>>,
    // This ram is used only for testing.
    test_ram: Option<[u8; 0x10000]>,
    dma_transfer: Option<u8>,
}

impl Bus {
    pub fn new(ppu: &Rc<RefCell<Ricoh2c02>>) -> Self {
        let mut bus = Bus {
            ram: Ram::new(),
            ppu: Rc::clone(ppu),
            cartridge: None,
            controller: Rc::new(RefCell::new(Controller::new())),
            test_ram: None,
            dma_transfer: None,
        };

        // Set up some fake RAM to use for testing purposes only.
        if cfg!(test) {
            bus.test_ram = Some([0; 0x10000]);
        }

        bus
    }

    pub fn controller(&self) -> Rc<RefCell<Controller>> {
        Rc::clone(&self.controller)
    }

    pub fn get_dma_transfer(&self) -> Option<u8> {
        self.dma_transfer
    }

    pub fn end_dma_transfer(&mut self) {
        self.dma_transfer = None;
    }

    pub fn load_cartridge(&mut self, cartridge: &Rc<RefCell<Cartridge>>) {
        self.cartridge = Some(Rc::clone(cartridge));
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => self.ram.write(address, data),
            0x2000..=0x3FFF => self.ppu.borrow_mut().cpu_write(address & 0x2007, data),
            0x4014 => self.dma_transfer = Some(data),
            0x4016 => self.controller().borrow_mut().latch(),
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

    pub fn cpu_read(&self, address: u16) -> u8 {
        match address {
            0x0..=0x1FFF => self.ram.read(address),
            0x2000..=0x3FFF => self.ppu.borrow_mut().cpu_read(address & 0x2007),
            0x4016 => self.controller().borrow_mut().read_button(),
            0x4020..=0xFFFF => match &self.cartridge {
                Some(cartridge) => cartridge.borrow().cpu_read(address),
                None => self.get_test_ram(address),
            },
            _ => self.get_test_ram(address),
        }
    }
}
