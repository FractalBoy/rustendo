pub struct Cartridge {
    raw: Vec<u8>,
}

enum Mirroring {
    Vertical,
    Horizontal,
}

enum ConsoleType {
    NES,
    NintendoVsSystem,
    NintendoPlaychoice10,
    ExtendedConsoleType,
}

impl Cartridge {
    pub fn new(raw: Vec<u8>) -> Self {
        Cartridge { raw }
    }

    pub fn header(&self) -> &[u8] {
        &self.raw[0..0x10]
    }

    fn calc_rom_size(size: u16) -> u16 {
        match size & 0xF00 >> 8 {
            0xF => {
                let multiplier = size & 0x3;
                let exponent = (size & 0xFC) >> 2;
                2u16.pow(exponent as u32) * (multiplier * 2 + 1)
            }
            _ => size * 0x4000,
        }
    }

    pub fn prg_rom_size(&self) -> usize {
        let lsb = self.header()[4] as u16;
        let msb = ((self.header()[9] as u16) & 0xF) << 4;
        let size = msb | lsb;

        Cartridge::calc_rom_size(size) as usize
    }

    pub fn prg_rom(&self) -> &[u8] {
        let start = 0x10 + self.trainer_size();
        let end = start + self.prg_rom_size();
        &self.raw[start..end]
    }

    pub fn chr_rom_size(&self) -> usize{
        let lsb = self.header()[5] as u16;
        let msb = (self.header()[9] as u16) & 0xF0;
        let size = msb | lsb;

        Cartridge::calc_rom_size(size) as usize
    }

    pub fn chr_rom(&self) -> &[u8] {
        let start = 0x10 + self.trainer_size() + self.prg_rom_size();
        let end = start + self.chr_rom_size();
        &self.raw[start..end]
    }

    pub fn has_trainer(&self) -> bool {
        self.header()[6] & 0x4 == 0x4
    }

    pub fn trainer(&self) -> &[u8] {
        let start:usize = 0x10;
        let end = 0x10 + self.trainer_size() as usize;
        &self.raw[start..end]
    }

    pub fn trainer_size(&self) -> usize {
        if self.has_trainer() {
            0x200
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cartridge;
    use std::fs;
    use std::path::Path;

    //    #[test]
    fn it_works() {
        let current_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let nes_test = current_dir.parent().unwrap().join("nestest.nes");
        let buffer = fs::read(nes_test).unwrap();

        let cartridge = Cartridge::new(buffer);
    }
}
