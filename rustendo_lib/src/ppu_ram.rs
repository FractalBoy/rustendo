use crate::cartridge::MirroringType;

pub struct Ram {
    nametables: [[u8; 0x400]; 2],
    mirroring: MirroringType,
}

impl Ram {
    pub fn new() -> Self {
        Ram {
            nametables: [[0; 0x400]; 2],
            mirroring: MirroringType::Vertical,
        }
    }

    pub fn set_mirroring_type(&mut self, mirroring: MirroringType) {
        self.mirroring = mirroring;
    }

    pub fn map_address(&self, address: u16) -> (usize, usize) {
        let address = address & 0x2FFF;

        match self.mirroring {
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
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        let (nametable, address) = self.map_address(address);
        self.nametables[nametable][address]
    }

    pub fn write(&mut self, address: u16, data: u8) {
        let (nametable, address) = self.map_address(address);
        self.nametables[nametable][address] = data;
    }
}
