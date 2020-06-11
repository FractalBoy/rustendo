use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::mos6502::Mos6502;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Nes {
    bus: Rc<RefCell<Bus>>,
    cpu: Mos6502,
}

impl Nes {
    pub fn new() -> Self {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let cpu = Mos6502::new(&bus);
        Nes { bus, cpu }
    }

    pub fn load_cartridge(&self, cartridge: Cartridge) {
        self.bus.borrow_mut().load_cartridge(cartridge);
    }

    pub fn clock(&mut self) {
        self.cpu.clock();
        self.bus.borrow_mut().ppu.clock();
    }
}
