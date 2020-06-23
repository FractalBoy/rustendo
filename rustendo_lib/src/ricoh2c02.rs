use crate::ppu_bus::Bus;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Copy, Clone)]
enum IncrementMode {
    AddOneGoingAcross = 0,
    AddThirtyTwoGoingDown = 1,
}

#[derive(Debug, Copy, Clone)]
enum SpriteSize {
    EightByEight = 0,
    EightBySixteen = 1,
}

#[derive(Debug, Copy, Clone)]
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

    pub fn get_background_pattern_table_address(&self) -> u16 {
        self.background_pattern_table_address
    }

    pub fn get_increment_mode(&self) -> IncrementMode {
        self.increment_mode
    }

    pub fn disable_nmi(&mut self) {
        self.nmi_enable = false;
    }

    pub fn get_nmi_enable(&self) -> bool {
        self.nmi_enable
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

    pub fn get_background_enable(&self) -> bool {
        self.background_enable
    }

    pub fn get_sprite_enable(&self) -> bool {
        self.sprite_enable
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

    pub fn set_vertical_blank_started(&mut self, data: bool) {
        self.vertical_blank_started = data;
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

struct Register {
    coarse_x_scroll: u8,
    coarse_y_scroll: u8,
    nametable_select: u8,
    fine_y_scroll: u8,
}

impl Register {
    pub fn new() -> Self {
        Register {
            coarse_x_scroll: 0,
            coarse_y_scroll: 0,
            nametable_select: 0,
            fine_y_scroll: 0,
        }
    }

    pub fn get_coarse_x_scroll(&self) -> u8 {
        self.coarse_x_scroll
    }

    pub fn get_coarse_y_scroll(&self) -> u8 {
        self.coarse_y_scroll
    }

    pub fn set_coarse_x_scroll(&mut self, data: u8) {
        self.coarse_x_scroll = (data & 0xF8) >> 3;
    }

    pub fn set_coarse_y_scroll(&mut self, data: u8) {
        self.coarse_y_scroll = (data & 0xF8) >> 3;
    }

    fn get_fine_y_scroll(&self) -> u8 {
        self.fine_y_scroll
    }

    fn get_nametable_select_x(&self) -> bool {
        self.nametable_select & 0x01 == 0x01
    }

    fn set_nametable_select_x(&mut self, data: bool) {
        self.nametable_select = (data as u8) | self.nametable_select & 0x02;
    }

    fn get_nametable_select_y(&self) -> bool {
        self.nametable_select & 0x02 == 0x02
    }

    fn set_nametable_select_y(&mut self, data: bool) {
        self.nametable_select = self.nametable_select & 0x01 | (data as u8) << 1
    }

    pub fn toggle_horizontal_nametable(&mut self) {
        self.set_nametable_select_x(!self.get_nametable_select_x());
    }

    pub fn toggle_vertical_nametable(&mut self) {
        self.set_nametable_select_y(!self.get_nametable_select_y());
    }

    pub fn copy_horizontal_address(&mut self, register: &Register) {
        self.set_coarse_x_scroll(register.coarse_x_scroll);
        self.set_nametable_select_x(register.get_nametable_select_x());
    }

    pub fn set_fine_y_scroll(&mut self, data: u8) {
        self.fine_y_scroll = data & 0x7;
    }

    pub fn set_address_high(&mut self, address: u8) {
        self.set((((address as u16) & 0x3F) << 8) | (self.get() & 0xFF))
    }

    pub fn set_address_low(&mut self, address: u8) {
        self.set((self.get() & 0x3F00) | ((address as u16) & 0xFF))
    }

    pub fn get_attribute_memory_offset(&self) -> u16 {
        // The attribute table for each nametable is
        // located at:
        //
        // Nametable 0: 23C0 = 0010 00 11 11 000 000
        // Nametable 1: 27C0 = 0010 01 11 11 000 000
        // Nametable 2: 2BC0 = 0010 10 11 11 000 000
        // Nametable 3: 2FC0 = 0010 11 11 11 000 000
        //                          ^^       ^^^ ^^^
        // attribute table select --||       ||| |||
        // y coordinate ---------------------||| |||
        // x coordinate -------------------------|||
        //
        // We use the nametable select (two bits) to select
        // the correct attribute table.
        //
        // Since each byte in the attribute table controls 4 bytes in the nametable,
        // we divide the coarse x and coarse y by 4 (by shifting right twice).
        //
        (self.nametable_select as u16) << 11
            | ((self.coarse_y_scroll as u16) >> 2) << 3
            | (self.coarse_x_scroll as u16) >> 2
    }

    pub fn get_nametable_offset(&self) -> u16 {
        self.get() | 0x0FFF
    }

    pub fn set(&mut self, data: u16) {
        self.coarse_x_scroll = (data & 0x1F) as u8;
        self.coarse_y_scroll = ((data & 0x3E0) >> 5) as u8;
        self.nametable_select = ((data & 0xC00) >> 10) as u8;
        self.fine_y_scroll = ((data & 0x7000) >> 12) as u8;
    }

    pub fn get(&self) -> u16 {
        ((self.fine_y_scroll as u16) << 12)
            | ((self.nametable_select as u16) << 10)
            | ((self.coarse_y_scroll as u16) << 5)
            | (self.coarse_x_scroll as u16)
    }

    pub fn copy(&mut self, register: &Register) {
        self.set(register.get());
    }

    pub fn increment(&mut self, increment: IncrementMode) {
        let increment = match increment {
            IncrementMode::AddOneGoingAcross => 1,
            IncrementMode::AddThirtyTwoGoingDown => 32,
        };

        self.set(self.get().wrapping_add(increment));
    }
}

pub struct Ricoh2c02 {
    bus: Rc<RefCell<Bus>>,
    primary_oam: [u8; 0x100],
    secondary_oam: [u8; 0x20],
    scanline: u32,
    cycle: u32,
    ppu_ctrl: PpuCtrl,
    ppu_mask: PpuMask,
    ppu_status: PpuStatus,
    oam_addr: u8,
    ppu_data: u8,
    vram_address: Register,
    temp_vram_address: Register,
    next_bg_tile_id: u8,
    next_bg_tile_attr: u8,
    next_bg_tile_msb: u16,
    next_bg_tile_lsb: u16,
    fine_x_scroll: u8,
    address_latch: bool,
    odd_frame: bool,
    palette: [(u8, u8, u8); 0x40],
    screen: [[(u8, u8, u8); 0x100]; 0xF0],
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
            primary_oam: [0; 0x100],
            secondary_oam: [0; 0x20],
            cycle: 0,
            scanline: 261,
            ppu_ctrl: PpuCtrl::new(),
            ppu_mask: PpuMask::new(),
            ppu_status: PpuStatus::new(),
            oam_addr: 0,
            ppu_data: 0,
            address_latch: false,
            odd_frame: false,
            vram_address: Register::new(),
            temp_vram_address: Register::new(),
            next_bg_tile_id: 0,
            next_bg_tile_attr: 0,
            next_bg_tile_msb: 0,
            next_bg_tile_lsb: 0,
            fine_x_scroll: 0,
            palette: Self::get_palette(),
            screen: [[(0, 0, 0); 0x100]; 0xF0],
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

    fn set_fine_x_scroll(&mut self, data: u8) {
        self.fine_x_scroll = data & 0x7;
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x2000 => 0,
            0x2001 => 0,
            0x2002 => {
                let data = self.ppu_status.get();
                // Clear bit 7
                self.ppu_status.set(self.ppu_status.get() & !0x80u8);
                // Clear address latch
                self.address_latch = false;
                // Top 3 bits with lower 5 bits
                // set to lower 5 bits of data buffer
                data & 0xE0 | self.ppu_data & 0x1F
            }
            0x2003 => 0,
            0x2004 => self.primary_oam[self.oam_addr as usize],
            0x2005 => 0,
            0x2006 => 0,
            0x2007 => {
                let address = self.vram_address.get();
                self.vram_address
                    .increment(self.ppu_ctrl.get_increment_mode());
                let ppu_data = self.ppu_data;
                self.ppu_data = self.ppu_read(address);

                // Palette range returns data immediately,
                // otherwise the data from the buffer is returned
                match address {
                    0x3F00..=0x3FFF => self.ppu_data,
                    _ => ppu_data,
                }
            }
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
                let address = self.oam_addr;
                self.oam_addr = self.oam_addr.wrapping_add(1);
                self.primary_oam[address as usize] = data;
            }
            0x2005 => {
                if !self.address_latch {
                    self.temp_vram_address.set_coarse_x_scroll(data);
                    self.set_fine_x_scroll(data);
                    self.address_latch = true;
                } else {
                    self.temp_vram_address.set_coarse_y_scroll(data);
                    self.temp_vram_address.set_fine_y_scroll(data);
                    self.address_latch = false;
                }
            }
            0x2006 => {
                if !self.address_latch {
                    self.temp_vram_address.set_address_high(data);
                    self.address_latch = true;
                } else {
                    self.temp_vram_address.set_address_low(data);
                    self.vram_address.copy(&self.temp_vram_address);
                    self.address_latch = false;
                }
            }
            0x2007 => {
                let address = self.vram_address.get();
                self.vram_address
                    .increment(self.ppu_ctrl.get_increment_mode());
                self.bus.borrow_mut().ppu_write(address, data);
            }
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
            // The remainder of memory mirrors the palette addresses
            0x3F20..=0x3FFF => self.ppu_read(address & 0x3F1F),
            _ => unreachable!(),
        }
    }

    pub fn oam_dma(&mut self, address: u16, data: u8) {
        let address = (address as u8).wrapping_add(self.oam_addr);
        self.primary_oam[address as usize] = data;
    }

    fn rendering_enabled(&self) -> bool {
        self.ppu_mask.get_background_enable() || self.ppu_mask.get_sprite_enable()
    }

    pub fn color_at_coord(&self, x: u32, y: u32) -> (u8, u8, u8) {
        self.screen[y as usize][x as usize]
    }

    fn update_next_bg_tile_id(&mut self) {
        self.next_bg_tile_id = self.ppu_read(0x2000 | self.vram_address.get_nametable_offset());
    }

    fn update_next_bg_tile_attr(&mut self) {
        self.next_bg_tile_attr =
            self.ppu_read(0x23C0 | self.vram_address.get_attribute_memory_offset());

        // The attribute tile is 1 byte and applies
        // to a 4-byte by 4-byte region of the nametable.
        //
        // It's arranged like this:
        // 7654 3210
        // |||| ||++- Color bits 3-2 for top left quadrant of this byte
        // |||| ++--- Color bits 3-2 for top right quadrant of this byte
        // ||++------ Color bits 3-2 for bottom left quadrant of this byte
        // ++-------- Color bits 3-2 for bottom right quadrant of this byte

        let bottom_right = self.next_bg_tile_attr & 0xC0 >> 6;
        let bottom_left = self.next_bg_tile_attr & 0x30 >> 4;
        let top_right = self.next_bg_tile_attr & 0x0C >> 2;
        let top_left = self.next_bg_tile_attr & 0x03;

        // This is the top left of the nametable (coarse_x, coarse_y):
        // (00,00) (01,00) | (10,00) (11,00)
        // (00,01) (01,01) | (10,01) (11,01)
        // ----------------|----------------
        // (00,10) (01,10) | (10,10) (11,10)
        // (00,01) (01,11) | (10,11) (11,11)
        //
        // To the bottom and right, the lower two bits
        // of the coordinate cycle.
        //
        // Top half: coarse_y & 0x2 == 0x0
        // Bottom half: coarse_y & 0x2 == 0x2
        // Left half: coarse_x & 0x2 == 0x0
        // Right half: coarse x & 0x2 == 0x2

        let coarse_x = self.vram_address.get_coarse_x_scroll();
        let coarse_y = self.vram_address.get_coarse_y_scroll();

        self.next_bg_tile_attr = match coarse_x & 0x2 {
            0x0 => match coarse_y & 0x2 {
                0x0 => top_left,
                0x2 => bottom_left,
                _ => unreachable!(),
            },
            0x2 => match coarse_y & 0x2 {
                0x0 => top_right,
                0x2 => bottom_right,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
    }

    fn update_next_bg_tile_lsb(&mut self) {
        self.next_bg_tile_lsb = self.ppu_ctrl.get_background_pattern_table_address()
            | (self.next_bg_tile_id as u16) << 4
            | 0 << 3
            | self.vram_address.get_fine_y_scroll() as u16;
    }

    fn update_next_bg_tile_msb(&mut self) {
        self.next_bg_tile_msb = self.ppu_ctrl.get_background_pattern_table_address()
            | (self.next_bg_tile_id as u16) << 4
            | 1 << 3
            | self.vram_address.get_fine_y_scroll() as u16;
    }

    fn increment_horizontal(&mut self) {
        // If we would scroll into the next nametable (because we exceed a width of 32 bytes)
        // Reset coarse X and switch to the next nametable (horizontally)
        //
        // If we're in the left nametable, this will go to the right, and vice versa

        if !self.rendering_enabled() {
            return;
        }

        if self.vram_address.get_coarse_x_scroll() == 0x1F {
            self.vram_address.set_coarse_x_scroll(0);
            self.vram_address.toggle_horizontal_nametable();
        } else {
            self.vram_address
                .set_coarse_x_scroll(self.vram_address.get_coarse_x_scroll() + 1);
        }
    }

    fn increment_vertical(&mut self) {
        if !self.rendering_enabled() {
            return;
        }

        if self.vram_address.get_fine_y_scroll() & 0x7 == 0x7 {
            // Fine Y's 3 bits are full, overflow to coarse Y
            self.vram_address.set_fine_y_scroll(0);

            if self.vram_address.get_coarse_y_scroll() == 29 {
                // Row 29 is the last row of tiles in a nametable.
                // To wrap to the next nametable when incrementing coarse Y from 29, the vertical nametable is switched by toggling bit 11, and coarse Y wraps to row 0.Row 29 is the last row of tiles in a nametable. To wrap to the next nametable when incrementing coarse Y from 29,
                // the vertical nametable is toggled and coarse Y wraps to row 0.
                self.vram_address.set_coarse_y_scroll(0);
                self.vram_address.toggle_vertical_nametable();
            } else if self.vram_address.get_coarse_y_scroll() == 31 {
                // Coarse Y can be set out of bounds,
                // which will cause the PPU to read the attribute data stored there
                // as tile data. If coarse Y is incremented from 31, it will wrap to 0,
                // but the nametable will not switch.
                //
                // For this reason, a write >= 240 to $2005 may appear as a "negative"
                // scroll value, where 1 or 2 rows of attribute data will appear
                // before the nametable's tile data is reached.
                //
                // Some games use this to move the top of the nametable
                // out of the Overscan area.
                self.vram_address.set_coarse_y_scroll(0);
            } else {
                // Coarse Y not at the bounds of the nametable; just add 1.
                self.vram_address
                    .set_coarse_y_scroll(self.vram_address.get_coarse_y_scroll() + 1);
            }
        } else {
            // Fine Y does not need to wrap; just add 1.
            self.vram_address
                .set_fine_y_scroll(self.vram_address.get_fine_y_scroll() + 1);
        }
    }

    pub fn clock(&mut self, nmi_enable: &mut bool) {
        match self.scanline {
            261 => {
                // Pre-render scanline, fill shift registers with data
                // for the first two tiles of the next scanline.

                // If rendering is enabled, set VRAM to temp VRAM for cycles 280 through 304
                if self.cycle >= 280 && self.cycle <= 304 && self.rendering_enabled() {
                    self.vram_address.copy(&self.temp_vram_address);
                }
            }
            0..=239 => {
                // Visible scanlines. Render both background and sprites.
                match self.cycle {
                    0 => {
                        // Idle cycle, unless it's an odd frame and rendering is enabled.
                        // If it's an odd frame, go directly to the next cycle.
                        if self.odd_frame && self.rendering_enabled() {
                            self.cycle += 1;
                            self.clock(nmi_enable);
                        }
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
                        match (self.cycle - 1) & 0x7 {
                            0 => self.update_next_bg_tile_id(),
                            1 => (),
                            2 => self.update_next_bg_tile_attr(),
                            3 => (),
                            4 => self.update_next_bg_tile_lsb(),
                            5 => (),
                            6 => self.update_next_bg_tile_msb(),
                            // At the end of each tile, increment right one.
                            7 => self.increment_horizontal(),
                            _ => unreachable!(),
                        }

                        // At the end of each screen, increment down one.
                        if self.cycle == 256 {
                            self.increment_vertical();
                        }
                    }
                    257..=320 => {
                        // The tile data for the sprites on the __next__ scanline are
                        // fetched here:
                        //
                        // 1. Garbage nametable byte
                        // 2. Garbage nametable byte
                        // 3. Pattern table tile low
                        // 4. Pattern table tile high (+8 bytes from pattern table tile low)

                        if self.cycle == 257 {
                            // Copy bits controlling horizontal position from temp VRAM to VRAM.
                            self.vram_address
                                .copy_horizontal_address(&self.temp_vram_address);
                        }

                        match (self.cycle - 1) & 0x7 {
                            0 => self.update_next_bg_tile_id(),
                            1 => (),
                            2 => self.update_next_bg_tile_attr(),
                            3 => (),
                            4 => self.update_next_bg_tile_lsb(),
                            5 => (),
                            6 => self.update_next_bg_tile_msb(),
                            // At the end of each tile, increment right one.
                            7 => self.increment_horizontal(),
                            _ => unreachable!(),
                        }
                    }
                    321..=336 => {
                        // The first two tiles for the next scanline are fetched here.
                        match (self.cycle - 1) & 0x7 {
                            0 => unimplemented!(),
                            1 => unimplemented!(),
                            2 => unimplemented!(),
                            3 => unimplemented!(),
                            4 => unimplemented!(),
                            5 => unimplemented!(),
                            6 => unimplemented!(),
                            7 => unimplemented!(),
                            _ => unreachable!(),
                        }
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
                        self.ppu_status.set_vertical_blank_started(true);

                        if self.ppu_ctrl.get_nmi_enable() {
                            *nmi_enable = true;
                            self.ppu_ctrl.disable_nmi();
                        }
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
            self.odd_frame = !self.odd_frame;
        }
    }
}
