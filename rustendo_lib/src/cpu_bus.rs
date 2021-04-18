use crate::cartridge::Cartridge;
use crate::controller::Controller;
use crate::cpu_ram::Ram;
use crate::ricoh2c02::Ricoh2c02;

pub struct Bus {
    ram: Box<Ram>,
    ppu: Box<Ricoh2c02>,
    controller: Controller,
    // This ram is used only for testing.
    test_ram: Option<[u8; 0x10000]>,
    dma_transfer: Option<u8>,
}

impl Bus {
    pub fn new(cartridge: Option<Box<Cartridge>>) -> Self {
        let mut bus = Bus {
            ram: Box::new(Ram::new()),
            ppu: Box::new(Ricoh2c02::new(cartridge)),
            controller: Controller::new(),
            test_ram: None,
            dma_transfer: None,
        };

        // Set up some fake RAM to use for testing purposes only.
        if cfg!(test) {
            bus.test_ram = Some([0; 0x10000]);
        }

        bus
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

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x0..=0x1FFF => self.ram.read(address),
            0x2000..=0x3FFF => self.ppu.cpu_read(address & 0x2007),
            0x4016 => self.controller.read_button(),
            0x4020..=0xFFFF => match &self.ppu.cartridge {
                Some(cartridge) => cartridge.cpu_read(address),
                None => self.get_test_ram(address)
            } 
            _ => self.get_test_ram(address),
        }
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => self.ram.write(address, data),
            0x2000..=0x3FFF => self.ppu.cpu_write(address & 0x2007, data),
            0x4014 => self.dma_transfer = Some(data),
            0x4016 => self.controller.latch(),
            0x4020..=0xFFFF => match &mut self.ppu.cartridge {
                Some(cartridge) => cartridge.cpu_write(address, data),
                None => self.set_test_ram(address, data)
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
}
