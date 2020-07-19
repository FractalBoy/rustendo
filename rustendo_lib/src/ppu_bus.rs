use crate::cartridge::{Cartridge, MirroringType};
use crate::ppu_ram::Ram;
use crate::ricoh2c02::Ricoh2c02;

pub struct Bus {
    ram: Box<Ram>,
    screen: Box<[[(u8, u8, u8); 0x100]; 0xF0]>,
    pub ppu: Option<Box<Ricoh2c02>>,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: Box::new(Ram::new()),
            screen: Box::new([[(0, 0, 0); 0x100]; 0xF0]),
            ppu: Some(Box::new(Ricoh2c02::new())),
        }
    }

    pub fn ppu_read(&self, cartridge: &Option<Cartridge>, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => match cartridge {
                Some(cartridge) => cartridge.ppu_read(address),
                None => 0,
            },
            0x2000..=0x3EFF => match cartridge {
                Some(cartridge) => self.ram.read(cartridge.mirroring_type(), address),
                None => self.ram.read(MirroringType::Vertical, address),
            },
            _ => 0,
        }
    }

    pub fn ppu_write(&mut self, cartridge: &mut Option<Cartridge>, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => match cartridge {
                Some(cartridge) => cartridge.ppu_write(address, data),
                None => (),
            },
            0x2000..=0x3EFF => match cartridge {
                Some(cartridge) => self.ram.write(cartridge.mirroring_type(), address, data),
                None => self.ram.write(MirroringType::Vertical, address, data),
            },
            _ => (),
        }
    }

    pub fn cpu_read(&mut self, cartridge: &Option<Cartridge>, address: u16) -> u8 {
        let mut data = 0;

        if let Some(mut ppu) = self.ppu.take() {
            data = ppu.cpu_read(self, cartridge, address);
            self.ppu = Some(ppu);
        }

        data
    }

    pub fn cpu_write(&mut self, cartridge: &mut Option<Cartridge>, address: u16, data: u8) {
        if let Some(mut ppu) = self.ppu.take() {
            ppu.cpu_write(self, cartridge, address, data);
            self.ppu = Some(ppu);
        }
    }

    pub fn clock(&mut self, cartridge: &mut Option<Cartridge>, nmi_enable: &mut bool) -> bool {
        let mut frame_complete = false;

        if let Some(mut ppu) = self.ppu.take() {
            frame_complete = ppu.clock(self, cartridge, nmi_enable);
            self.ppu = Some(ppu);
        }

        frame_complete
    }

    pub fn get_screen(&self) -> Box<[[(u8, u8, u8); 0x100]; 0xF0]> {
        if let Some(ppu) = &self.ppu {
            ppu.get_screen()
        } else {
            self.screen.clone()
        }
    }

    pub fn oam_dma(&mut self, address: u16, data: u8) {
        if let Some(ppu) = &mut self.ppu {
            ppu.oam_dma(address, data);
        }
    }
}
