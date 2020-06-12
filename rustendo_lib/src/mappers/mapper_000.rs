use super::Mapper;

pub struct Mapper000 {}

impl Mapper000 {
    pub fn new() -> Self {
        Mapper000 {}
    }
}

impl Mapper for Mapper000 {
    fn cpu_read(&self, address: u16) -> u16 {
        match address {
            0x4020..=0x7FFF => 0,
            0x8000..=0xBFFF => 0,
            _ => 0,
        }
    }

    fn cpu_write(&self, address: u16) -> u16 {
        unimplemented!()
    }

    fn ppu_read(&self, address: u16) -> u16 {
        unimplemented!()
    }

    fn ppu_write(&self, address: u16) -> u16 {
        unimplemented!()
    }
}
