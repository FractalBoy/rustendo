pub trait Mapper {
    fn cpu_read(&self, address: u16) -> (Option<usize>, Option<u8>);
    fn cpu_write(&mut self, address: u16, data: u8) -> Option<usize>;
    fn ppu_read(&self, address: u16) -> (Option<usize>, Option<u8>);
    fn ppu_write(&mut self, address: u16, data: u8) -> Option<usize>;
}

pub mod mapper_000;
pub mod mapper_001;
