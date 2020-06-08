use crate::bus::Bus;
use crate::mos6502::Mos6502;
use crate::nes2c02::Nes2c02;
use crate::ram::Ram;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Nes {
    cpu: Mos6502,
    bus: Rc<RefCell<Bus>>,
}

impl Nes {
    pub fn new() -> Self {
        let mut bus = Bus::new();
        let ram = Box::new(Ram::new());
        let ppu = Box::new(Nes2c02::new());
        bus.connect(ram);
        bus.connect(ppu);
        let bus = Rc::new(RefCell::new(bus));
        let cpu = Mos6502::new(&bus);

        Nes {
            cpu,
            bus,
        }
    }

    pub fn clock(&mut self) {
        // Clock the CPU and all the devices on the bus on every clock.
        // The devices will divide the clock themselves.
        self.bus.borrow_mut().clock();
        self.cpu.clock();
    }
}

#[cfg(test)]
mod tests {
    use super::Nes;

    #[test]
    fn it_works() {
        let nes = Nes::new();
    }
}
