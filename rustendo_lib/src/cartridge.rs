pub struct Cartridge {
    raw: Vec<u8>,
    header: [u8; 0x10],
    trainer: Option<[u8; 0x200]>,
    prg: Vec<u8>,
    chr: Vec<u8>,
    inst_rom: Option<[u8; 0x2000]>,
    p_rom: Option<[u8; 0x20]>,
    mirroring: Mirroring,
}

enum Mirroring {
    Vertical,
    Horizontal,
}

enum ConsoleType {
    NES,
    NintendoVsSystem,
    NintendoPlaychoice10,
    ExtendedConsoleType
}

impl Cartridge {
    pub fn new(raw: Vec<u8>) -> Self {
        // NES 2.0 has bit 2 cleared and bit 4 set
        if raw[7] & 0xC == 0x8 {
            Cartridge::read_nes_2_0(raw)
        } else {
            Cartridge::read_ines(raw)
        }
    }

    fn read_ines(raw: Vec<u8>) -> Self {
        unimplemented!();
    }

    fn read_nes_2_0(raw: Vec<u8>) -> Self {
        let prg_rom_size = raw[4];
        let chr_rom_size = raw[5];
        let mirroring = if raw[6] & 0x1 == 0x1 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };
        let battery = raw[6] & 0x2 == 0x2;
        let trainer = raw[6] & 0x4 == 0x4;
        let four_screen_mode = raw[6] & 0x8 == 0x8;
        let mapper_number = raw[6] & 0xF0;
        let console_type = match raw[7] & 0x3 {
            0x0 => ConsoleType::NES,
            0x1 => ConsoleType::NintendoVsSystem,
            0x2 => ConsoleType::NintendoPlaychoice10,
            0x3 => ConsoleType::ExtendedConsoleType,
            t => panic!("{} is not a valid console type", t)
        };
        unimplemented!();
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
