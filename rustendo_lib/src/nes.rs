use crate::cartridge::Cartridge;
use crate::controller::Controller;
use crate::cpu_bus::Bus as CpuBus;

pub struct Nes {
    bus: CpuBus,
    cartridge: Option<Cartridge>,
    clocks: u32,
    dma_cycle: u16,
    dma_data: u8,
    dma_dummy: bool,
}

impl Nes {
    pub fn new() -> Self {
        Nes {
            bus: CpuBus::new(),
            cartridge: None,
            clocks: 0,
            dma_cycle: 0,
            dma_data: 0,
            dma_dummy: true,
        }
    }

    pub fn controller(&mut self) -> &mut Controller {
        self.bus.controller()
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = Some(cartridge);
    }

    pub fn clock(&mut self) -> bool {
        let mut nmi_enable = false;

        // PPU runs at 1/4 the master clock speed
        let frame_complete =
            self.bus
                .ppu_bus
                .clock(&mut self.cartridge, &mut nmi_enable);

        // CPU runs at 1/12 the master clock speed, 3x as slow as the PPU
        if self.clocks % 3 == 0 {
            let dma_transfer = self.bus.get_dma_transfer();

            match dma_transfer {
                Some(data) => self.dma_transfer(data),
                None => {
                    self.bus.clock(&mut self.cartridge);
                }
            }
        }

        if nmi_enable {
            self.bus.cpu.nmi();
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

    pub fn get_screen(&self) -> [[(u8, u8, u8); 0x100]; 0xF0] {
        self.ppu.borrow().get_screen()
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }
}

// #[cfg(test)]
// mod tests {
//     use super::Nes;
//     use crate::cartridge::Cartridge;
//     use std::fs;
//     use std::path::Path;

//     #[test]
//     fn it_works() {
//         let current_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
//         let nes_test = current_dir.parent().unwrap().join("colors.nes");
//         let buffer = fs::read(nes_test).unwrap();

//         let mut nes = Nes::new();
//         let cartridge = Cartridge::new(buffer);
//         nes.load_cartridge(cartridge);

//         nes.reset();
//         loop {
//             nes.clock();
//         }
//     }
// }
