use crate::bus::Connect;

pub struct Ram {
    ram: [u8; 0x800],
}

impl Connect for Ram {
    fn read(&self, address_high: u8, address_low: u8) -> Option<u8> {
        let address = ((address_high as u16) << 8) | (address_low as u16);

        match self.find_address(address) {
            Some(address) => Some(self.ram[address as usize]),
            None => None,
        }
    }

    fn write(&mut self, address_high: u8, address_low: u8, data: u8) {
        let address = ((address_high as u16) << 8) | (address_low as u16);

        match self.find_address(address) {
            Some(address) => self.ram[address as usize] = data,
            None => return,
        }
    }

    fn clock(&mut self) {}
}

impl Ram {
    pub fn new() -> Self {
        Ram { ram: [0; 0x800] }
    }

    fn find_address(&self, address: u16) -> Option<u16> {
        match address {
            0x0..=0x7FF => Some(address),
            0x800..=0xFFF => Some(address - 0x800),
            0x1000..=0x17FF => Some(address - 0x1000),
            0x1800..=0x1FFF => Some(address - 0x1800),
            _ => None,
        }
    }

    pub fn load_mem(&mut self, memory: &[u8]) {
        let mut ram = vec![0; 0x800];
        ram.splice(0..memory.len(), memory.iter().cloned());
        self.ram.copy_from_slice(&ram);
    }
}
