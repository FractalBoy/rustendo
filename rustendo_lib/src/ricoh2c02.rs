use crate::ppu_bus::Bus;
use bitvec::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

enum IncrementMode {
    AddOneGoingAcross = 0,
    AddThirtyTwoGoingDown = 1,
}

enum SpriteSize {
    EightByEight = 0,
    EightBySixteen = 1,
}

enum PpuSelect {
    ReadBackdrop = 0,
    OutputColor = 1,
}

struct PpuCtrl {
    base_nametable_address: u16,
    increment_mode: IncrementMode,
    sprite_pattern_table_address: u16,
    background_pattern_table_address: u16,
    sprite_size: SpriteSize,
    ppu_select: PpuSelect,
    nmi_enable: bool,
}

impl PpuCtrl {
    pub fn new() -> Self {
        PpuCtrl {
            base_nametable_address: 0x2000,
            increment_mode: IncrementMode::AddOneGoingAcross,
            sprite_pattern_table_address: 0x0000,
            background_pattern_table_address: 0x0000,
            sprite_size: SpriteSize::EightByEight,
            ppu_select: PpuSelect::ReadBackdrop,
            nmi_enable: false,
        }
    }

    pub fn set(&mut self, byte: u8) {
        self.base_nametable_address = match byte & 0x03 {
            0x0 => 0x2000,
            0x1 => 0x2400,
            0x2 => 0x2800,
            0x3 => 0x2C00,
            _ => unreachable!(),
        };

        self.increment_mode = if byte & 0x04 == 0x04 {
            IncrementMode::AddThirtyTwoGoingDown
        } else {
            IncrementMode::AddOneGoingAcross
        };

        self.sprite_pattern_table_address = if byte & 0x08 == 0x08 { 0x1000 } else { 0x0000 };

        self.background_pattern_table_address = if byte & 0x10 == 0x10 { 0x1000 } else { 0x0000 };

        self.sprite_size = if byte & 0x20 == 0x20 {
            SpriteSize::EightBySixteen
        } else {
            SpriteSize::EightByEight
        };

        self.ppu_select = if byte & 0x40 == 0x40 {
            PpuSelect::OutputColor
        } else {
            PpuSelect::ReadBackdrop
        };

        self.nmi_enable = byte & 0x80 == 0x80;
    }

    pub fn get(&self) -> u8 {
        let mut byte = match self.base_nametable_address {
            0x2000 => 0x0,
            0x2400 => 0x1,
            0x2800 => 0x2,
            0x2C00 => 0x3,
            _ => unreachable!(),
        };

        byte |= match self.increment_mode {
            IncrementMode::AddThirtyTwoGoingDown => 0x04,
            IncrementMode::AddOneGoingAcross => 0x00,
        };

        byte |= match self.sprite_pattern_table_address {
            0x0000 => 0x00,
            0x1000 => 0x08,
            _ => unreachable!(),
        };

        byte |= match self.background_pattern_table_address {
            0x0000 => 0x00,
            0x1000 => 0x10,
            _ => unreachable!(),
        };

        byte |= match self.sprite_size {
            SpriteSize::EightByEight => 0x00,
            SpriteSize::EightBySixteen => 0x20,
        };

        byte |= match self.ppu_select {
            PpuSelect::ReadBackdrop => 0x00,
            PpuSelect::OutputColor => 0x40,
        };

        byte |= if self.nmi_enable { 0x80 } else { 0x00 };

        byte
    }
}

struct PpuMask {
    greyscale: bool,
    background_left_column_enable: bool,
    sprite_left_column_enable: bool,
    background_enable: bool,
    sprite_enable: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,
}

impl PpuMask {
    pub fn new() -> Self {
        PpuMask {
            greyscale: false,
            background_left_column_enable: false,
            sprite_left_column_enable: false,
            background_enable: false,
            sprite_enable: false,
            emphasize_red: false,
            emphasize_green: false,
            emphasize_blue: false,
        }
    }

    pub fn set(&mut self, byte: u8) {
        self.greyscale = byte & 0x01 == 0x01;
        self.background_left_column_enable = byte & 0x02 == 0x02;
        self.sprite_left_column_enable = byte & 0x04 == 0x04;
        self.background_enable = byte & 0x08 == 0x08;
        self.sprite_enable = byte & 0x10 == 0x10;
        self.emphasize_red = byte & 0x20 == 0x20;
        self.emphasize_green = byte & 0x40 == 0x40;
        self.emphasize_blue = byte & 0x80 == 0x80;
    }

    pub fn get(&self) -> u8 {
        self.greyscale as u8
            | (self.background_left_column_enable as u8) << 1
            | (self.sprite_left_column_enable as u8) << 2
            | (self.background_enable as u8) << 3
            | (self.sprite_enable as u8) << 4
            | (self.emphasize_red as u8) << 5
            | (self.emphasize_green as u8) << 6
            | (self.emphasize_blue as u8) << 7
    }
}

struct PpuStatus {
    sprite_overflow: bool,
    sprite_zero_hit: bool,
    vertical_blank_started: bool,
}

impl PpuStatus {
    pub fn new() -> Self {
        PpuStatus {
            sprite_overflow: false,
            sprite_zero_hit: false,
            vertical_blank_started: false,
        }
    }

    pub fn set(&mut self, byte: u8) {
        let byte = byte >> 5;
        self.sprite_overflow = byte & 0x01 == 0x01;
        self.sprite_zero_hit = byte & 0x02 == 0x02;
        self.vertical_blank_started = byte & 0x04 == 0x04;
    }

    pub fn get(&self) -> u8 {
        (self.sprite_overflow as u8
            | (self.sprite_zero_hit as u8) << 1
            | (self.vertical_blank_started as u8) << 2)
            << 5
    }
}

pub struct Ricoh2c02 {
    bus: Rc<RefCell<Bus>>,
    oam: [u8; 0x100],
    clocks: u32,
    scanline: u32,
    cycle: u32,
    ppu_ctrl: PpuCtrl,
    ppu_mask: PpuMask,
    ppu_status: PpuStatus,
    oam_addr: u8,
    oam_data: u8,
    ppu_scroll: u8,
    ppu_addr: u8,
    ppu_data: u8,
    oam_dma: u8,
    palette: [(u8, u8, u8); 0x40],
    universal_background_color: u8,
    background_palette_0: (u8, u8, u8),
    background_palette_1: (u8, u8, u8),
    background_palette_2: (u8, u8, u8),
    background_palette_3: (u8, u8, u8),
    sprite_palette_0: (u8, u8, u8),
    sprite_palette_1: (u8, u8, u8),
    sprite_palette_2: (u8, u8, u8),
    sprite_palette_3: (u8, u8, u8),
}

const CYCLES_PER_SCANLINE: u32 = 341;
const SCANLINES_PER_FRAME: u32 = 262;

impl Ricoh2c02 {
    pub fn new(bus: &Rc<RefCell<Bus>>) -> Self {
        Ricoh2c02 {
            bus: Rc::clone(bus),
            oam: [0; 0x100],
            clocks: 0,
            cycle: 0,
            scanline: 261,
            ppu_ctrl: PpuCtrl::new(),
            ppu_mask: PpuMask::new(),
            ppu_status: PpuStatus::new(),
            oam_addr: 0,
            oam_data: 0,
            ppu_scroll: 0,
            ppu_addr: 0,
            ppu_data: 0,
            oam_dma: 0,
            palette: Self::get_palette(),
            universal_background_color: 0,
            background_palette_0: (0, 0, 0),
            background_palette_1: (0, 0, 0),
            background_palette_2: (0, 0, 0),
            background_palette_3: (0, 0, 0),
            sprite_palette_0: (0, 0, 0),
            sprite_palette_1: (0, 0, 0),
            sprite_palette_2: (0, 0, 0),
            sprite_palette_3: (0, 0, 0),
        }
    }

    fn get_palette() -> [(u8, u8, u8); 0x40] {
        [
            (84, 84, 84),
            (0, 30, 116),
            (8, 16, 144),
            (48, 0, 136),
            (68, 0, 100),
            (92, 0, 48),
            (84, 4, 0),
            (60, 24, 0),
            (32, 42, 0),
            (8, 58, 0),
            (0, 64, 0),
            (0, 60, 0),
            (0, 50, 60),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (152, 150, 152),
            (8, 76, 196),
            (48, 50, 236),
            (92, 30, 228),
            (136, 20, 176),
            (160, 20, 100),
            (152, 34, 32),
            (120, 60, 0),
            (84, 90, 0),
            (40, 114, 0),
            (8, 124, 0),
            (0, 118, 40),
            (0, 102, 120),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (236, 238, 236),
            (76, 154, 236),
            (120, 124, 236),
            (176, 98, 236),
            (228, 84, 236),
            (236, 88, 180),
            (236, 106, 100),
            (212, 136, 32),
            (160, 170, 0),
            (116, 196, 0),
            (76, 208, 32),
            (56, 204, 108),
            (56, 180, 204),
            (60, 60, 60),
            (0, 0, 0),
            (0, 0, 0),
            (236, 238, 236),
            (168, 204, 236),
            (188, 188, 236),
            (212, 178, 236),
            (236, 174, 236),
            (236, 174, 212),
            (236, 180, 176),
            (228, 196, 144),
            (204, 210, 120),
            (180, 222, 120),
            (168, 226, 144),
            (152, 226, 180),
            (160, 214, 228),
            (160, 162, 160),
            (0, 0, 0),
            (0, 0, 0),
        ]
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x2000 => 0,
            0x2001 => 0,
            0x2002 => {
                // Clear bit 7
                self.ppu_status.set(self.ppu_status.get() & !0x80u8);
                self.ppu_status.get()
            }
            0x2003 => 0,
            0x2004 => self.oam_data,
            0x2005 => 0,
            0x2006 => 0,
            0x2007 => self.ppu_data,
            _ => 0,
        }
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            0x2000 => self.ppu_ctrl.set(data),
            0x2001 => self.ppu_mask.set(data),
            0x2002 => self.ppu_status.set(data),
            0x2003 => self.oam_addr = data,
            0x2004 => {
                self.oam_addr = self.oam_addr.wrapping_add(1);
                self.oam_data = data
            }
            0x2005 => self.ppu_scroll = data,
            0x2006 => self.ppu_addr = data,
            0x2007 => self.ppu_data = data,
            _ => (),
        }
    }

    pub fn ppu_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3EFF => self.bus.borrow().ppu_read(address),
            0x3F00 | 0x3F04 | 0x3F08 | 0x3F0C | 0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
                self.universal_background_color
            }
            0x3F01..=0x3F03 => match address & 0x0003 {
                0x0001 => self.background_palette_0.0,
                0x0002 => self.background_palette_0.1,
                0x0003 => self.background_palette_0.2,
                _ => unreachable!(),
            },
            0x3F05..=0x3F07 => match address & 0x0003 {
                0x0001 => self.background_palette_1.0,
                0x0002 => self.background_palette_1.1,
                0x0003 => self.background_palette_1.2,
                _ => unreachable!(),
            },
            0x3F09..=0x3F0B => match address & 0x0003 {
                0x0001 => self.background_palette_2.0,
                0x0002 => self.background_palette_2.1,
                0x0003 => self.background_palette_2.2,
                _ => unreachable!(),
            },
            0x3F0D..=0x3F0F => match address & 0x0003 {
                0x0001 => self.background_palette_3.0,
                0x0002 => self.background_palette_3.1,
                0x0003 => self.background_palette_3.2,
                _ => unreachable!(),
            },
            0x3F11..=0x3F13 => match address & 0x0003 {
                0x0001 => self.sprite_palette_0.0,
                0x0002 => self.sprite_palette_0.1,
                0x0003 => self.sprite_palette_0.2,
                _ => unreachable!(),
            },
            0x3F15..=0x3F17 => match address & 0x0003 {
                0x0001 => self.sprite_palette_1.0,
                0x0002 => self.sprite_palette_1.1,
                0x0003 => self.sprite_palette_1.2,
                _ => unreachable!(),
            },
            0x3F19..=0x3F1B => match address & 0x0003 {
                0x0001 => self.sprite_palette_2.0,
                0x0002 => self.sprite_palette_2.1,
                0x0003 => self.sprite_palette_2.2,
                _ => unreachable!(),
            },
            0x3F1D..=0x3F1F => match address & 0x0003 {
                0x0001 => self.sprite_palette_3.0,
                0x0002 => self.sprite_palette_3.1,
                0x0003 => self.sprite_palette_3.2,
                _ => unreachable!(),
            },
            _ => 0,
        }
    }

    pub fn clock(&mut self) {
        // Divide input clock by four.
        if self.clocks % 4 == 0 {
            self.do_next_cycle();
        }

        self.clocks = self.clocks.wrapping_add(1);
    }

    pub fn do_next_cycle(&mut self) {
        match self.scanline {
            261 => {
                // Pre-render scanline, fill shift registers with data
                // for the first two tiles of the next scanline.
            }
            0..=239 => {
                // Visible scanlines. Render both background and sprites.
                match self.cycle {
                    0 => {
                        // Idle cycle.
                    }
                    1..=256 => {
                        // The data for each tile is fetched during this phase.
                        // Each memory access takes two PPU cycles to complete,
                        // and 4 must be performed per tile:
                        //
                        // 1. Nametable byte
                        // 2. Attribute table byte
                        // 3. Pattern table tile low
                        // 4. Pattern table tile high (+8 bytes from pattern table tile low)
                    }
                    257..=320 => {
                        // The tile data for the sprites on the __next__ scanline are
                        // fetched here:
                        //
                        // 1. Garbage nametable byte
                        // 2. Garbage nametable byte
                        // 3. Pattern table tile low
                        // 4. Pattern table tile high (+8 bytes from pattern table tile low)
                    }
                    321..=336 => {
                        // The first two tiles for the next scanline are fetched here.
                    }
                    337..=340 => {
                        // Two bytes are fetched, but the purpose for this is unknown.
                        //
                        // 1. Nametable byte
                        // 2. Nametable byte
                    }
                    _ => (),
                }
            }
            240 => {
                // Post-render scanline. The PPU just idles during this scanline.
                // The VBlank flag isn't set until after this scanline.
            }
            241..=260 => {
                // Vertical blank.
                match self.cycle {
                    1 => {
                        // VBlank flag set here. VBlank NMI also occurs here.
                    }
                    // Idle otherwise.
                    _ => (),
                }
            }
            _ => (),
        }

        self.cycle += 1;

        if self.cycle == CYCLES_PER_SCANLINE {
            self.scanline += 1;
            self.cycle = 0;
        }
        if self.scanline == SCANLINES_PER_FRAME {
            self.scanline = 0;
        }
    }
}
