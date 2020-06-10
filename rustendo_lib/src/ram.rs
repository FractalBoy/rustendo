pub struct Ram {
    ram: [u8; 0x800],
}

impl Ram {
    pub fn new() -> Self {
        Ram { ram: [0; 0x800] }
    }

    fn find_address(&self, address: u16) -> usize {
        (address as usize) & 0x7FF
    }

    pub fn read(&self, address: u16) -> u8 {
        self.ram[self.find_address(address)]
    }

    pub fn write(&mut self, address: u16, data: u8) {
        self.ram[self.find_address(address)] = data
    }

    pub fn load_mem(&mut self, memory: &[u8]) {
        let mut ram = vec![0; 0x800];
        ram.splice(0..memory.len(), memory.iter().cloned());
        self.ram.copy_from_slice(&ram);
    }
}
