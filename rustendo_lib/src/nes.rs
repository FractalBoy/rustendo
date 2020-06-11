use crate::cartridge::Cartridge;
use crate::cpu_bus::Bus as CpuBus;
use crate::mos6502::Mos6502;
use crate::ppu_bus::Bus as PpuBus;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Nes {
    cpu_bus: Rc<RefCell<CpuBus>>,
    ppu_bus: Rc<RefCell<PpuBus>>,
    cpu: Mos6502,
}

impl Nes {
    pub fn new() -> Self {
        let cpu_bus = Rc::new(RefCell::new(CpuBus::new()));
        let ppu_bus = Rc::new(RefCell::new(PpuBus::new()));
        let cpu = Mos6502::new(&cpu_bus);
        Nes {
            cpu,
            cpu_bus,
            ppu_bus,
        }
    }

    pub fn load_cartridge(&self, cartridge: Cartridge) {
        let cartridge = Rc::new(RefCell::new(cartridge));
        self.cpu_bus.borrow_mut().load_cartridge(&cartridge);
        self.ppu_bus.borrow_mut().load_cartridge(&cartridge);
    }

    pub fn clock(&mut self) {
        self.cpu.clock();
        self.cpu_bus.borrow_mut().ppu.clock();
    }
}
