use crate::cartridge::MirroringType;

pub struct Ram {
    nametables: [[u8; 0x400]; 2],
}

impl Ram {
    pub fn new() -> Self {
        Ram {
            nametables: [[0; 0x400]; 2],
        }
    }

    pub fn map_address(&self, mirroring: MirroringType, address: u16) -> (usize, usize) {
        let address = address & 0x2FFF;

        match mirroring {
            MirroringType::Horizontal => match address {
                0x2000..=0x23FF => (0, (address & 0x3FF) as usize),
                0x2400..=0x27FF => (0, (address & 0x3FF) as usize),
                0x2800..=0x2BFF => (1, (address & 0x3FF) as usize),
                0x2C00..=0x2FFF => (1, (address & 0x3FF) as usize),
                _ => unreachable!(),
            },
            MirroringType::Vertical => match address {
                0x2000..=0x23FF => (0, (address & 0x3FF) as usize),
                0x2400..=0x27FF => (1, (address & 0x3FF) as usize),
                0x2800..=0x2BFF => (0, (address & 0x3FF) as usize),
                0x2C00..=0x2FFF => (1, (address & 0x3FF) as usize),
                _ => unreachable!(),
            },
            MirroringType::OneScreen => (0, (address & 0x3FF) as usize),
        }
    }

    pub fn read(&self, mirroring: MirroringType, address: u16) -> u8 {
        let (nametable, address) = self.map_address(mirroring, address);
        self.nametables[nametable][address]
    }

    pub fn write(&mut self, mirroring: MirroringType, address: u16, data: u8) {
        let (nametable, address) = self.map_address(mirroring, address);
        self.nametables[nametable][address] = data;
    }
}
