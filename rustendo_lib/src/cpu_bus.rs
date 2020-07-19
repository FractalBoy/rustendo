use crate::cartridge::Cartridge;
use crate::controller::Controller;
use crate::cpu_ram::Ram;
use crate::ppu_bus::Bus as PpuBus;
use crate::mos6502::Mos6502;

pub struct Bus {
    ram: Ram,
    cpu: Option<Mos6502>,
    pub ppu_bus: PpuBus,
    controller: Controller,
    // This ram is used only for testing.
    test_ram: Option<[u8; 0x10000]>,
    dma_transfer: Option<u8>,
}

impl Bus {
    pub fn new() -> Self {
        let mut bus = Bus {
            ram: Ram::new(),
            cpu: Some(Mos6502::new()),
            ppu_bus: PpuBus::new(),
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

    pub fn clock(&mut self, cartridge: &mut Option<Cartridge>) -> bool {
        let mut instruction_complete = false;

        if let Some(mut cpu) = self.cpu.take() {
            instruction_complete = cpu.clock(self, cartridge);
            self.cpu = Some(cpu);
        }

        instruction_complete
    }

    pub fn reset(&mut self) {
        if let Some(cpu) = &mut self.cpu {
            cpu.reset();
        }
    }

    pub fn nmi(&mut self) {
        if let Some(cpu) = &mut self.cpu {
            cpu.nmi();
        }
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

    pub fn cpu_write(&mut self, cartridge: &mut Option<Cartridge>, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => self.ram.write(address, data),
            0x2000..=0x3FFF => self.ppu_bus.cpu_write(cartridge, address & 0x2007, data),
            0x4014 => self.dma_transfer = Some(data),
            0x4016 => self.controller.latch(),
            0x4020..=0xFFFF => match cartridge {
                Some(mapper) => mapper.cpu_write(address, data),
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

    pub fn cpu_read(&mut self, cartridge: &Option<Cartridge>, address: u16) -> u8 {
        match address {
            0x0..=0x1FFF => self.ram.read(address),
            0x2000..=0x3FFF => self.ppu_bus.cpu_read(cartridge, address & 0x2007),
            0x4016 => self.controller.read_button(),
            0x4020..=0xFFFF => match cartridge {
                Some(cartridge) => cartridge.cpu_read(address),
                None => self.get_test_ram(address),
            },
            _ => self.get_test_ram(address),
        }
    }
}
