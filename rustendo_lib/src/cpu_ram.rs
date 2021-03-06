pub struct Ram {
    ram: Vec<u8>,
}

impl Ram {
    pub fn new() -> Self {
        Ram { ram: vec![0; 0x800] }
    }

    fn find_address(&self, address: u16) -> usize {
        (address as usize) & 0x7FF
    }

    pub fn read(&self, address: u16) -> u8 {
        self.ram[self.find_address(address)]
    }

    pub fn write(&mut self, address: u16, data: u8) {
        let address = self.find_address(address);
        self.ram[address] = data;
    }
}

#[cfg(test)]
mod tests {
    use super::Ram;

    #[test]
    fn it_works() {
        // Right now, this test does nothing - it just silences warnings.
        let mut ram = Ram::new();
        ram.read(0);
        ram.write(0, 0);
    }
}
