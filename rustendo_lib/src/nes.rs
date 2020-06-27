use crate::cartridge::Cartridge;
use crate::cpu_bus::Bus as CpuBus;
use crate::mos6502::Mos6502;
use crate::ppu_bus::Bus as PpuBus;
use crate::ricoh2c02::Ricoh2c02;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Nes {
    cpu_bus: Rc<RefCell<CpuBus>>,
    ppu_bus: Rc<RefCell<PpuBus>>,
    cpu: Mos6502,
    ppu: Rc<RefCell<Ricoh2c02>>,
    clocks: u32,
    dma_cycle: u16,
    dma_data: u8,
    dma_dummy: bool,
}

impl Nes {
    pub fn new() -> Self {
        let ppu_bus = Rc::new(RefCell::new(PpuBus::new()));
        let ppu = Rc::new(RefCell::new(Ricoh2c02::new(&ppu_bus)));
        let cpu_bus = Rc::new(RefCell::new(CpuBus::new(&ppu)));
        let cpu = Mos6502::new(&cpu_bus);
        Nes {
            cpu,
            ppu,
            cpu_bus,
            ppu_bus,
            clocks: 0,
            dma_cycle: 0,
            dma_data: 0,
            dma_dummy: true,
        }
    }

    pub fn load_cartridge(&self, cartridge: Cartridge) {
        let cartridge = Rc::new(RefCell::new(cartridge));
        self.cpu_bus.borrow_mut().load_cartridge(&cartridge);
        self.ppu_bus.borrow_mut().load_cartridge(&cartridge);
    }

    pub fn clock(&mut self) -> bool {
        let mut nmi_enable = false;

        // PPU runs at 1/4 the master clock speed
        let frame_complete = self.ppu.borrow_mut().clock(&mut nmi_enable);

        // CPU runs at 1/12 the master clock speed, 3x as slow as the PPU
        if self.clocks % 3 == 0 {
            let dma_transfer = self.cpu_bus.borrow().get_dma_transfer();

            match dma_transfer {
                Some(data) => self.dma_transfer(data),
                None => {
                    self.cpu.clock();
                }
            }
        }

        if nmi_enable {
            self.cpu.nmi();
        }

        self.clocks = self.clocks.wrapping_add(1);
        frame_complete
    }

    fn dma_transfer(&mut self, data: u8) {
        let starting_addr = (data as u16) << 8;
        let current_addr = starting_addr + self.dma_cycle;

        // Don't start the DMA transfer until we hit an odd clock cycle.
        if self.dma_dummy {
            if self.clocks % 2 == 1 {
                self.dma_dummy = false;
            }

            return;
        }

        if self.clocks % 2 == 0 {
            self.dma_data = self.cpu_bus.borrow().cpu_read(current_addr);
        } else {
            self.ppu.borrow_mut().oam_dma(self.dma_cycle, self.dma_data);
            self.dma_cycle = self.dma_cycle.wrapping_add(1);
        }

        // End the DMA transfer after 256 bytes are copied.
        if self.dma_cycle == 0x100 {
            self.cpu_bus.borrow_mut().end_dma_transfer();
            self.dma_dummy = true;
            self.dma_cycle = 0;
        }
    }

    pub fn color_at_coord(&self, x: u32, y: u32) -> (u8, u8, u8) {
        self.ppu.borrow().color_at_coord(x, y)
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }
}

//#[cfg(test)]
//mod tests {
//    use super::Nes;
//    use crate::cartridge::Cartridge;
//    use std::fs;
//    use std::path::Path;
//
//    #[test]
//    fn it_works() {
//        let current_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
//        let nes_test = current_dir.parent().unwrap().join("nestest.nes");
//        let buffer = fs::read(nes_test).unwrap();
//
//        let mut nes = Nes::new();
//        let cartridge = Cartridge::new(buffer);
//        nes.load_cartridge(cartridge);
//
//        nes.reset();
//        loop {
//            nes.clock();
//        }
//    }
//}
