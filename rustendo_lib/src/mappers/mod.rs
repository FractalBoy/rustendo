pub trait Mapper {
    fn cpu_read(&self, address: u16) -> (Option<u16>, Option<u8>);
    fn cpu_write(&mut self, address: u16, data: u8) -> Option<u16>;
    fn ppu_read(&self, address: u16) -> (Option<u16>, Option<u8>);
    fn ppu_write(&mut self, address: u16, data: u8) -> Option<u16>;
}

pub mod mapper_000;
