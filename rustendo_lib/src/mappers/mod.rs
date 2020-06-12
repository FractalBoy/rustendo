pub trait Mapper {
    fn cpu_read(&self, address: u16) -> u16;
    fn cpu_write(&self, address: u16) -> u16;
    fn ppu_read(&self, address: u16) -> u16;
    fn ppu_write(&self, address: u16) -> u16;
}

pub mod mapper_000;
