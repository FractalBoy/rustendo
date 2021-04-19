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
