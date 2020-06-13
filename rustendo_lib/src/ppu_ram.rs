use crate::cartridge::MirroringType;

pub struct Ram {
    ram: [u8; 0x800],
    mirroring: MirroringType,
}

impl Ram {
    pub fn new(mirroring: MirroringType) -> Self {
        Ram {
            ram: [0; 0x800],
            mirroring,
        }
    }

    pub fn map_address(&self, address: u16) -> u16 {
        let address = address & 0x1FFF;
        match self.mirroring {
            MirroringType::Horizontal => match address {
                0x0000..=0x03FF => address,
                0x0400..=0x07FF => address & 0x03FF,
                0x0800..=0x0BFF => address,
                0x0C00..=0x0FFF => address & 0x0BFF,
                _ => address
            },
            MirroringType::Vertical => match address {
                0x0000..=0x03FF => address,
                0x0400..=0x07FF => address,
                0x0800..=0x0BFF => address & 0x07FF,
                0x0C00..=0x1000 => address & 0x07FF,
                _ => address
            },
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.ram[self.map_address(address) as usize]
    }

    pub fn write(&mut self, address: u16, data: u8) {
        self.ram[self.map_address(address) as usize] = data;
    }
}
