use crate::cartridge::MirroringType;

pub struct Ram {
    ram: [u8; 0x800],
    mirroring: MirroringType,
}

impl Ram {
    pub fn new() -> Self {
        Ram {
            ram: [0; 0x800],
            mirroring: MirroringType::Vertical,
        }
    }

    pub fn set_mirroring_type(&mut self, mirroring: MirroringType) {
        self.mirroring = mirroring;
    }

    pub fn map_address(&self, address: u16) -> u16 {
        let address = match address {
            // 0x3000 - 0x3EFF mirrors back to 0x2000 - 0x2EFF
            0x3000..=0x3EFF => return self.map_address(address & 0x2FFF),
            _ => address,
        };

        let address = match self.mirroring {
            MirroringType::Horizontal => match address {
                0x2000..=0x23FF => address,
                0x2400..=0x27FF => address & 0x23FF,
                0x2800..=0x2BFF => address - 0x400,
                0x2C00..=0x2FFF => (address & 0x2BFF) - 0x400,
                _ => address,
            },
            MirroringType::Vertical => match address {
                0x2000..=0x23FF => address,
                0x2400..=0x27FF => address,
                0x2800..=0x2BFF => address & 0x27FF,
                0x2C00..=0x2FFF => address & 0x27FF,
                _ => address,
            },
        };

        address & 0x1FFF
    }

    pub fn read(&self, address: u16) -> u8 {
        self.ram[self.map_address(address) as usize]
    }

    pub fn write(&mut self, address: u16, data: u8) {
        self.ram[self.map_address(address) as usize] = data;
    }
}
