use crate::cartridge::Cartridge;
use crate::controller::Controller;
use crate::cpu_ram::Ram;
use crate::ricoh2c02::Ricoh2c02;

pub struct Bus {
    ram: Ram,
    ppu: Ricoh2c02,
    controller: Controller,
    #[cfg(test)]
    test_ram: Vec<u8>,
    dma_transfer: Option<u8>,
}

impl Bus {
    #[cfg(not(test))]
    pub fn new() -> Self {
        Bus {
            ram: Ram::new(),
            ppu: Ricoh2c02::new(),
            controller: Controller::new(),
            dma_transfer: None,
        }
    }

    #[cfg(test)]
    pub fn new() -> Self {
        Bus {
            ram: Ram::new(),
            ppu: Ricoh2c02::new(),
            controller: Controller::new(),
            dma_transfer: None,
            test_ram: vec![0; 0x10000],
        }
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.ppu.load_cartridge(cartridge);
    }

    pub fn ppu_clock(&mut self, nmi_enable: &mut bool) -> bool {
        self.ppu.clock(nmi_enable)
    }

    pub fn get_ppu(&self) -> &Ricoh2c02 {
        &self.ppu
    }

    pub fn get_ppu_mut(&mut self) -> &mut Ricoh2c02 {
        &mut self.ppu
    }

    pub fn controller(&mut self) -> &mut Controller {
        &mut self.controller
    }

    pub fn get_dma_transfer(&self) -> Option<u8> {
        self.dma_transfer
    }

    pub fn end_dma_transfer(&mut self) {
        self.dma_transfer = None;
    }

    #[cfg(not(test))]
    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x0..=0x1FFF => self.ram.read(address),
            0x2000..=0x3FFF => self.ppu.cpu_read(address & 0x2007),
            0x4016 => self.controller.read_button(),
            0x4020..=0xFFFF => {
                if self.ppu.has_cartridge() {
                    self.ppu.cartridge_cpu_read(address)
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    #[cfg(test)]
    pub fn cpu_read(&mut self, address: u16) -> u8 {
        self.test_ram[address as usize]
    }

    #[cfg(not(test))]
    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => self.ram.write(address, data),
            0x2000..=0x3FFF => self.ppu.cpu_write(address & 0x2007, data),
            0x4014 => self.dma_transfer = Some(data),
            0x4016 => self.controller.latch(),
            0x4020..=0xFFFF => {
                if self.ppu.has_cartridge() {
                    self.ppu.cartridge_cpu_write(address, data)
                } else {
                    ()
                }
            }
            _ => (),
        };
    }

    #[cfg(test)]
    pub fn cpu_write(&mut self, address: u16, data: u8) {
        self.test_ram[address as usize] = data;
    }
}
