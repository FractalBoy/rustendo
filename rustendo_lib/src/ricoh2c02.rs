use crate::cartridge::Cartridge;
use crate::ppu_bus::Bus;
use std::ops::Index;
use std::ops::IndexMut;

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
    nametable_select: u8,
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
            nametable_select: 0x00,
            increment_mode: IncrementMode::AddOneGoingAcross,
            sprite_pattern_table_address: 0x0000,
            background_pattern_table_address: 0x0000,
            sprite_size: SpriteSize::EightByEight,
            ppu_select: PpuSelect::ReadBackdrop,
            nmi_enable: false,
        }
    }

    pub fn get_nametable_select(&self) -> u8 {
        self.nametable_select
    }

    pub fn get_background_pattern_table_address(&self) -> u16 {
        self.background_pattern_table_address
    }

    pub fn get_increment_mode(&self) -> IncrementMode {
        self.increment_mode
    }

    pub fn get_nmi_enable(&self) -> bool {
        self.nmi_enable
    }

    pub fn get_sprite_height(&self) -> u8 {
        match self.sprite_size {
            SpriteSize::EightByEight => 8,
            SpriteSize::EightBySixteen => 16,
        }
    }

    pub fn set(&mut self, byte: u8) {
        self.nametable_select = byte & 0x03;

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

    pub fn get_greyscale(&self) -> bool {
        self.greyscale
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

    pub fn get_vertical_blank_started(&self) -> bool {
        self.vertical_blank_started
    }

    pub fn set_sprite_overflow(&mut self, data: bool) {
        self.sprite_overflow = data;
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

enum RegisterBits {
    CoarseX = 0b000_00_00000_11111,
    CoarseY = 0b000_00_11111_00000,
    NametableSelect = 0b000_11_00000_00000,
    NametableSelectX = 0b000_01_00000_00000,
    NametableSelectY = 0b000_10_00000_00000,
    FineY = 0b111_00_00000_00000,
    AddressHigh = 0b00111111_00000000,
    AddressLow = 0b00000000_11111111,
}

bitfield!(Register, RegisterBits, u16);

impl Register {
    pub fn toggle_horizontal_nametable(&mut self) {
        self.set_field(
            RegisterBits::NametableSelectX,
            !self.get_field(RegisterBits::NametableSelectX),
        );
    }

    pub fn toggle_vertical_nametable(&mut self) {
        self.set_field(
            RegisterBits::NametableSelectY,
            !self.get_field(RegisterBits::NametableSelectY),
        );
    }

    pub fn copy_horizontal_address(&mut self, register: &Register) {
        self.set_field(
            RegisterBits::CoarseX,
            register.get_field(RegisterBits::CoarseX),
        );
        self.set_field(
            RegisterBits::NametableSelectX,
            register.get_field(RegisterBits::NametableSelectX),
        );
    }

    pub fn copy_vertical_address(&mut self, register: &Register) {
        self.set_field(
            RegisterBits::CoarseY,
            register.get_field(RegisterBits::CoarseX),
        );
        self.set_field(
            RegisterBits::NametableSelectY,
            register.get_field(RegisterBits::NametableSelectX),
        );
        self.set_field(RegisterBits::FineY, register.get_field(RegisterBits::FineY));
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
        let nametable_select = self.get_field(RegisterBits::NametableSelect) as u16;
        let coarse_x = (self.get_field(RegisterBits::CoarseX) as u16) >> 2;
        let coarse_y = (self.get_field(RegisterBits::CoarseY) as u16) >> 2;

        nametable_select << 11 | coarse_y << 3 | coarse_x
    }

    pub fn get_nametable_offset(&self) -> u16 {
        self.register & 0x0FFF
    }

    pub fn copy(&mut self, register: &Register) {
        self.register = register.register
    }

    pub fn increment(&mut self, increment: IncrementMode) {
        let increment = match increment {
            IncrementMode::AddOneGoingAcross => 1,
            IncrementMode::AddThirtyTwoGoingDown => 32,
        };

        self.register = self.register.wrapping_add(increment);
    }
}

struct Sprite {
    pub top_y_position: u8,
    pub tile_id: u8,
    pub attributes: u8,
    pub left_x_position: u8,
}

impl Sprite {
    fn _in_range(scanline: u32, height: u8, byte: u8) -> bool {
        // We are evaluating the next scanline.
        let scanline = scanline + 1;
        let byte: u32 = byte.into();
        let height: u32 = height.into();

        scanline >= byte && scanline < byte + height
    }
    pub fn in_range(&self, scanline: u32, height: u8) -> bool {
        Self::_in_range(scanline, height, self.top_y_position)
    }

    pub fn in_range_with_sprite_overflow_bug(
        &self,
        scanline: u32,
        height: u8,
        byte: usize,
    ) -> bool {
        Self::_in_range(scanline, height, self[byte])
    }
}

impl Index<usize> for Sprite {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        match index & 0x3 {
            0 => &self.top_y_position,
            1 => &self.tile_id,
            2 => &self.attributes,
            3 => &self.left_x_position,
            _ => unreachable!(),
        }
    }
}

impl IndexMut<usize> for Sprite {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        match index & 0x3 {
            0 => &mut self.top_y_position,
            1 => &mut self.tile_id,
            2 => &mut self.attributes,
            3 => &mut self.left_x_position,
            _ => unreachable!(),
        }
    }
}

struct Oam {
    oam: Vec<u8>,
    shift_registers: Vec<u8>,
    counters: Vec<i16>,
    num_sprites: usize,
}

impl Oam {
    pub fn new(num_sprites: usize) -> Self {
        Oam {
            oam: vec![0; num_sprites * 4],
            shift_registers: vec![0; num_sprites],
            counters: vec![0; num_sprites],
            num_sprites: 0,
        }
    }

    pub fn get_sprite(&self, sprite: usize) -> Sprite {
        let start = sprite * 4;
        let end = (sprite + 1) * 4;
        let slice = &self.oam[start..end];

        Sprite {
            top_y_position: slice[0],
            tile_id: slice[1],
            attributes: slice[2],
            left_x_position: slice[3],
        }
    }

    pub fn copy_sprite(&mut self, oam: &Oam, sprite_num: usize) {
        if self.is_full() {
            return;
        }

        let sprite = oam.get_sprite(sprite_num);
        self.oam[self.num_sprites] = sprite.top_y_position;
        self.oam[self.num_sprites + 1] = sprite.tile_id;
        self.oam[self.num_sprites + 2] = sprite.attributes;
        self.oam[self.num_sprites + 3] = sprite.left_x_position;

        self.num_sprites += 1;
    }

    pub fn copy(&mut self, oam: &Oam) {
        self.oam.copy_from_slice(&oam.oam);

        // Initialize the counters, reset the shifters
        for sprite in 0..self.oam.len() {
            self.counters[sprite] = self.get_sprite(sprite).left_x_position.into();
            self.shift_registers[sprite] = 0;
        }
    }

    pub fn is_full(&self) -> bool {
        self.num_sprites == self.oam.len() / 4
    }

    pub fn decrement_counters(&mut self) {
        for counter in &mut self.counters {
            *counter -= 1;
        }
    }

    pub fn shift_sprites_into_registers(&mut self) {
        for sprite in 0..self.counters.len() {
            // Only shift the ones in range.
            if self.counters[sprite] <= 0 && self.counters[sprite] > -8 {
                let register = &mut self.shift_registers[sprite];
                // TODO: replace "bit" with next bit to shift into register
                let bit = 1;
                *register = (bit << 7) | ((*register >> 1) & 0x7F);
            }
        }
    }
}

impl Index<usize> for Oam {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.oam[index]
    }
}

impl IndexMut<usize> for Oam {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.oam[index]
    }
}

pub struct Ricoh2c02 {
    primary_oam: Oam,
    secondary_oam: Oam,
    current_scanline_oam: Oam,
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
    next_bg_tile_msb: u8,
    next_bg_tile_lsb: u8,
    bg_tile_msb_shifter: u16,
    bg_tile_lsb_shifter: u16,
    bg_attr_msb_shifter: u16,
    bg_attr_lsb_shifter: u16,
    fine_x_scroll: u8,
    address_latch: bool,
    odd_frame: bool,
    palette: [(u8, u8, u8); 0x40],
    screen: [[(u8, u8, u8); 0x100]; 0xF0],
    palette_ram: [u8; 0x20],
    current_sprite_number: u8,
    current_sprite_byte: u8,
}

const CYCLES_PER_SCANLINE: u32 = 341;
const SCANLINES_PER_FRAME: u32 = 262;

impl Ricoh2c02 {
    pub fn new() -> Self {
        Ricoh2c02 {
            primary_oam: Oam::new(64),
            secondary_oam: Oam::new(8),
            current_scanline_oam: Oam::new(8),
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
            bg_tile_msb_shifter: 0,
            bg_tile_lsb_shifter: 0,
            bg_attr_msb_shifter: 0,
            bg_attr_lsb_shifter: 0,
            fine_x_scroll: 0,
            palette: Self::get_palette(),
            screen: [[(0, 0, 0); 0x100]; 0xF0],
            palette_ram: [0; 0x20],
            current_sprite_number: 0,
            current_sprite_byte: 0,
        }
    }

    pub fn get_screen(&self) -> &[[(u8, u8, u8); 0x100]; 0xF0] {
        &self.screen
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

    pub fn cpu_read(&mut self, bus: &Bus, cartridge: &Option<Cartridge>, address: u16) -> u8 {
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
                let address = *self.vram_address;
                self.vram_address
                    .increment(self.ppu_ctrl.get_increment_mode());
                let ppu_data = self.ppu_data;
                self.ppu_data = self.ppu_read(bus, cartridge, address);

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

    pub fn cpu_write(&mut self, bus: &mut Bus, cartridge: &mut Option<Cartridge>, address: u16, data: u8) {
        match address {
            0x2000 => {
                self.ppu_ctrl.set(data);
                self.temp_vram_address.set_field(
                    RegisterBits::NametableSelect,
                    self.ppu_ctrl.get_nametable_select(),
                );
            }
            0x2001 => self.ppu_mask.set(data),
            0x2002 => self.ppu_status.set(data),
            0x2003 => self.oam_addr = data,
            0x2004 => {
                let address = self.oam_addr;

                // Only allow writes to OAM when PPU is not rendering.
                if self.rendering_enabled() && !self.ppu_status.get_vertical_blank_started() {
                    return;
                }

                self.primary_oam[address as usize] = data;
            }
            0x2005 => {
                if !self.address_latch {
                    self.temp_vram_address
                        .set_field(RegisterBits::CoarseX, data >> 3);
                    self.set_fine_x_scroll(data);
                    self.address_latch = true;
                } else {
                    self.temp_vram_address
                        .set_field(RegisterBits::CoarseY, data >> 3);
                    self.temp_vram_address.set_field(RegisterBits::FineY, data);
                    self.address_latch = false;
                }
            }
            0x2006 => {
                if !self.address_latch {
                    self.temp_vram_address
                        .set_field(RegisterBits::AddressHigh, data);
                    self.address_latch = true;
                } else {
                    self.temp_vram_address
                        .set_field(RegisterBits::AddressLow, data);
                    self.vram_address.copy(&self.temp_vram_address);
                    self.address_latch = false;
                }
            }
            0x2007 => {
                let address = *self.vram_address;
                self.vram_address
                    .increment(self.ppu_ctrl.get_increment_mode());
                self.ppu_write(bus, cartridge, address, data);
            }
            _ => (),
        }
    }

    pub fn ppu_read(&self, bus: &Bus, cartridge: &Option<Cartridge>, address: u16) -> u8 {
        // When the grayscale bit is set in the PPU mask,
        // only the top 2 bits of the palette are used, meaning only gray colors
        // are used: 0x00 (dark gray), 0x10 (light gray), 0x20 (white), 0x30 (white).
        let palette_mask = if self.ppu_mask.get_greyscale() {
            0x30
        } else {
            0xFF
        };

        match address {
            0x0000..=0x3EFF => bus.ppu_read(cartridge, address),
            0x3F00..=0x3FFF => match address & 0x1F {
                0x10 | 0x14 | 0x18 | 0x1C => {
                    self.palette_ram[(address & 0x0F) as usize] & palette_mask
                }
                address => self.palette_ram[address as usize] & palette_mask,
            },
            _ => 0,
        }
    }

    pub fn ppu_write(&mut self, bus: &mut Bus, cartridge: &mut Option<Cartridge>, address: u16, data: u8) {
        match address {
            0x0000..=0x3EFF => bus.ppu_write(cartridge, address, data),
            0x3F00..=0x3FFF => match address & 0x1F {
                0x10 | 0x14 | 0x18 | 0x1C => self.palette_ram[(address & 0x0F) as usize] = data,
                address => self.palette_ram[address as usize] = data,
            },
            _ => (),
        }
    }

    pub fn oam_dma(&mut self, address: u16, data: u8) {
        let address = (address as u8).wrapping_add(self.oam_addr);
        self.primary_oam[address as usize] = data;
    }

    fn rendering_enabled(&self) -> bool {
        self.ppu_mask.get_background_enable() || self.ppu_mask.get_sprite_enable()
    }

    fn update_next_bg_tile_id(&mut self, bus: &Bus, cartridge: &Option<Cartridge>) {
        self.next_bg_tile_id = self.ppu_read(
            bus,
            cartridge,
            0x2000 | self.vram_address.get_nametable_offset(),
        );
    }

    fn update_next_bg_tile_attr(&mut self, bus: &Bus, cartridge: &Option<Cartridge>) {
        self.next_bg_tile_attr = self.ppu_read(
            bus,
            cartridge,
            0x23C0 | self.vram_address.get_attribute_memory_offset(),
        );
        // The attribute tile is 1 byte and applies
        // to a 4-byte by 4-byte region of the nametable.
        //
        // It's arranged like this:
        // 7654 3210
        // |||| ||++- Color bits 3-2 for top left quadrant of this byte
        // |||| ++--- Color bits 3-2 for top right quadrant of this byte
        // ||++------ Color bits 3-2 for bottom left quadrant of this byte
        // ++-------- Color bits 3-2 for bottom right quadrant of this byte
        //
        // This is the top left of the nametable (coarse_x, coarse_y):
        // (00,00) (01,00) | (10,00) (11,00)
        // (00,01) (01,01) | (10,01) (11,01)
        // ----------------|----------------
        // (00,10) (01,10) | (10,10) (11,10)
        // (00,01) (01,11) | (10,11) (11,11)
        //
        // The top half has coarse_y = 0 or 1, the bottom half has coarse_y = 2 or 3
        // The left half has coarse_x = 0 or 1, the right half has coarse_x = 2 or 3
        let coarse_x = self.vram_address.get_field(RegisterBits::CoarseX);
        let coarse_y = self.vram_address.get_field(RegisterBits::CoarseY);

        // First, get the top or bottom:
        // * Top is lower nibble of attribute byte and coarse_y & 0x02 == 0x00
        // * Bottom is upper nibble of attribute byte and coarse_y & 0x02 == 0x02
        if coarse_y & 0x02 == 0x02 {
            self.next_bg_tile_attr >>= 4;
        }
        // Then, get the left or right
        // * Left is lower two bits of nibble and coarse_x & 0x02 == 0x00
        // * Right is upper two bits of nibble and coarse_x & 0x02 == 0x00
        if coarse_x & 0x02 == 0x02 {
            self.next_bg_tile_attr >>= 2;
        }
        self.next_bg_tile_attr &= 0x03;
    }

    //PPU addresses within the pattern tables can be decoded as follows:
    //
    // DCBA98 76543210
    // ---------------
    // 0HRRRR CCCCPTTT
    // |||||| |||||+++- T: Fine Y offset, the row number within a tile
    // |||||| ||||+---- P: Bit plane (0: "lower"; 1: "upper")
    // |||||| ++++----- C: Tile column
    // ||++++---------- R: Tile row
    // |+-------------- H: Half of sprite table (0: "left"; 1: "right")
    // +--------------- 0: Pattern table is at $0000-$1FFF
    //
    fn update_next_bg_tile_lsb(&mut self, bus: &Bus, cartridge: &Option<Cartridge>) {
        self.next_bg_tile_lsb = self.ppu_read(
            bus,
            cartridge,
            self.ppu_ctrl.get_background_pattern_table_address()
                | (self.next_bg_tile_id as u16) << 4
                | 0 << 3
                | self.vram_address.get_field(RegisterBits::FineY) as u16,
        );
    }

    fn update_next_bg_tile_msb(&mut self, bus: &Bus, cartridge: &Option<Cartridge>) {
        self.next_bg_tile_msb = self.ppu_read(
            bus,
            cartridge,
            self.ppu_ctrl.get_background_pattern_table_address()
                | (self.next_bg_tile_id as u16) << 4
                | 1 << 3
                | self.vram_address.get_field(RegisterBits::FineY) as u16,
        );
    }

    fn increment_horizontal(&mut self) {
        // If we would scroll into the next nametable (because we exceed a width of 32 bytes)
        // Reset coarse X and switch to the next nametable (horizontally)
        //
        // If we're in the left nametable, this will go to the right, and vice versa

        if !self.rendering_enabled() {
            return;
        }

        let coarse_x = self.vram_address.get_field(RegisterBits::CoarseX);

        if coarse_x == 0x1F {
            self.vram_address.set_field(RegisterBits::CoarseX, 0);
            self.vram_address.toggle_horizontal_nametable();
        } else {
            self.vram_address
                .set_field(RegisterBits::CoarseX, coarse_x + 1);
        }
    }

    fn increment_vertical(&mut self) {
        if !self.rendering_enabled() {
            return;
        }

        let fine_y = self.vram_address.get_field(RegisterBits::FineY);
        let coarse_y = self.vram_address.get_field(RegisterBits::CoarseY);

        if fine_y & 0x7 == 0x7 {
            // Fine Y's 3 bits are full, overflow to coarse Y
            self.vram_address.set_field(RegisterBits::FineY, 0);

            if coarse_y == 29 {
                // Row 29 is the last row of tiles in a nametable.
                // To wrap to the next nametable when incrementing coarse Y from 29,
                // the vertical nametable is toggled and coarse Y wraps to row 0.
                self.vram_address.set_field(RegisterBits::CoarseY, 0);
                self.vram_address.toggle_vertical_nametable();
            } else if coarse_y == 31 {
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
                self.vram_address.set_field(RegisterBits::CoarseY, 0);
            } else {
                // Coarse Y not at the bounds of the nametable; just add 1.
                self.vram_address
                    .set_field(RegisterBits::CoarseY, coarse_y + 1);
            }
        } else {
            // Fine Y does not need to wrap; just add 1.
            self.vram_address.set_field(RegisterBits::FineY, fine_y + 1);
        }
    }

    fn load_background_shifters(&mut self) {
        self.bg_tile_lsb_shifter = self.bg_tile_lsb_shifter & 0xFF00 | self.next_bg_tile_lsb as u16;
        self.bg_tile_msb_shifter = self.bg_tile_msb_shifter & 0xFF00 | self.next_bg_tile_msb as u16;

        // The background attribute shifters are actually 1 byte wide,
        // but they are for the lower byte of the tile shifters, so we'll use
        // 2 bytes for convenience, and shift them over 1 byte the same as the tile shifters.
        self.bg_attr_lsb_shifter = self.bg_attr_lsb_shifter & 0xFF00
            | (((self.next_bg_tile_attr as u16 & 0x01 == 0x01) as u16) * 0xFF);
        self.bg_attr_msb_shifter = self.bg_attr_msb_shifter & 0xFF00
            | (((self.next_bg_tile_attr as u16 & 0x02 == 0x02) as u16) * 0xFF);
    }

    fn update_background_shifters(&mut self) {
        if self.ppu_mask.get_background_enable() {
            self.bg_tile_lsb_shifter <<= 1;
            self.bg_tile_msb_shifter <<= 1;
            self.bg_attr_lsb_shifter <<= 1;
            self.bg_attr_msb_shifter <<= 1;
        }
    }

    fn calculate_pixel(&self, bus: &Bus, cartridge: &Option<Cartridge>) -> (u8, u8, u8) {
        let (pixel, palette) = if self.ppu_mask.get_background_enable() {
            let mask = 0x8000 >> self.fine_x_scroll;

            let pixel_lsb = self.bg_tile_lsb_shifter & mask == mask;
            let pixel_msb = self.bg_tile_msb_shifter & mask == mask;
            let pixel = (pixel_msb as u16) << 1 | pixel_lsb as u16;

            let attr_lsb = self.bg_attr_lsb_shifter & mask == mask;
            let attr_msb = self.bg_attr_msb_shifter & mask == mask;
            let palette = (attr_msb as u16) << 1 | (attr_lsb as u16);
            (pixel, palette)
        } else {
            (0, 0)
        };

        self.palette[(self.ppu_read(bus, cartridge, 0x3F00 | palette << 2 | pixel) & 0x3F) as usize]
    }

    pub fn update_background(&mut self, bus: &Bus, cartridge: &Option<Cartridge>) {
        match (self.cycle - 1) & 0x7 {
            0 => {
                if self.cycle >= 9 {
                    self.load_background_shifters();
                }
                self.update_next_bg_tile_id(bus, cartridge);
            }
            1 => (),
            2 => self.update_next_bg_tile_attr(bus, cartridge),
            3 => (),
            4 => self.update_next_bg_tile_lsb(bus, cartridge),
            5 => (),
            6 => self.update_next_bg_tile_msb(bus, cartridge),
            // At the end of each tile, increment right one.
            7 => self.increment_horizontal(),
            _ => unreachable!(),
        }
    }

    fn visible_scanline(&mut self, bus: &Bus, cartridge: &Option<Cartridge>) {
        self.update_background_shifters();
        self.update_background(bus, cartridge);

        if self.cycle == 256 {
            self.increment_vertical();
        }
    }

    fn sprite_evaluation(&mut self) {
        match self.scanline {
            1..=239 | 261 => match self.cycle {
                // Initialize secondary OAM to 0xFF, alternating reads and writes
                1..=64 => match self.cycle % 2 {
                    0 => self.secondary_oam[(self.cycle / 2) as usize] = 0xFF,
                    // Fake reads on odd cycles
                    1 => return,
                    _ => unreachable!(),
                },
                65..=256 => {
                    if self.cycle == 65 {
                        self.current_sprite_number = 0;
                        self.current_sprite_byte = 0;
                    }

                    // Don't do anything on fake odd read cycles
                    if self.cycle % 2 != 0 {
                        return;
                    }

                    // If the current sprite byte is non-zero, we're currently copying a
                    // sprite into secondary OAM
                    if self.current_sprite_byte > 0 {
                        self.current_sprite_byte += 1;

                        if self.current_sprite_byte == 4 {
                            // If we hit 4 bytes, we're onto the next sprite.
                            self.current_sprite_byte = 0;
                            self.current_sprite_number += 1;
                        } else if !self.secondary_oam.is_full() {
                            // If we haven't hit 4 bytes yet, we're still on the current sprite.
                            // We've already done the work to copy the sprite on the 0th byte,
                            // so we can just return.
                            //
                            // We don't return when secondary OAM is full to allow sprite overflow
                            // evaluation to still occur.
                            return;
                        }
                    }

                    // All 64 sprites have been evaluated.
                    if self.current_sprite_number == 64 {
                        return;
                    }

                    let next_sprite = self
                        .primary_oam
                        .get_sprite(self.current_sprite_number.into());

                    if self.secondary_oam.is_full() {
                        // Evaluate sprite overflow, with bug that treats the
                        // other parts of the sprite as the top Y position.
                        if next_sprite.in_range_with_sprite_overflow_bug(
                            self.scanline,
                            self.ppu_ctrl.get_sprite_height(),
                            self.current_sprite_byte.into(),
                        ) {
                            self.current_sprite_byte += 1;
                            self.ppu_status.set_sprite_overflow(true);
                        } else {
                            self.current_sprite_number += 1;
                            // This is the sprite overflow bug.
                            // The byte should really not be incremented.
                            self.current_sprite_byte += 1;
                        }
                    }

                    // If we get here, secondary OAM is not full
                    // and we still have sprites to evaluate.
                    if next_sprite.in_range(self.scanline, self.ppu_ctrl.get_sprite_height()) {
                        self.secondary_oam
                            .copy_sprite(&self.primary_oam, self.current_sprite_number.into());
                        self.current_sprite_byte = 1;
                    } else {
                        // Not in range. Move onto the next one...
                        self.current_sprite_number += 1;
                    }
                }
                340 => {
                    // Copy the entire secondary OAM into the OAM to be used for the next scanline.
                    self.current_scanline_oam.copy(&self.secondary_oam);
                }
                _ => (),
            },
            _ => {}
        }
    }

    pub fn clock(
        &mut self,
        bus: &Bus,
        cartridge: &Option<Cartridge>,
        nmi_enable: &mut bool,
    ) -> bool {
        if self.scanline == 0 && self.cycle == 0 && self.odd_frame && self.rendering_enabled() {
            // Idle cycle, unless it's an odd frame and rendering is enabled.
            // If it's an odd frame, go directly to the next cycle.
            self.cycle = 1;
        }

        if self.scanline == 261 && self.cycle == 1 {
            self.ppu_status.set_vertical_blank_started(false);
        }

        match self.scanline {
            0..=239 | 261 => match self.cycle {
                1..=256 | 321..=337 => self.visible_scanline(bus, cartridge),
                257 => {
                    self.load_background_shifters();
                    if self.rendering_enabled() {
                        self.vram_address
                            .copy_horizontal_address(&self.temp_vram_address);
                    }
                }
                280..=304 => {
                    if self.scanline == 261 && self.rendering_enabled() {
                        self.vram_address
                            .copy_vertical_address(&self.temp_vram_address);
                    }
                }
                // Garbage nametable bytes
                338 | 340 => self.update_next_bg_tile_id(bus, cartridge),
                _ => (),
            },
            241 => match self.cycle {
                1 => {
                    // VBlank flag set here. VBlank NMI also occurs here.
                    self.ppu_status.set_vertical_blank_started(true);

                    if self.ppu_ctrl.get_nmi_enable() {
                        *nmi_enable = true;
                    }
                }
                _ => (),
            },
            _ => (),
        }

        // Sprite evaluation and rendering
        self.sprite_evaluation();
        self.current_scanline_oam.decrement_counters();
        self.current_scanline_oam.shift_sprites_into_registers();

        if self.cycle < 256 && self.scanline < 240 {
            self.screen[self.scanline as usize][self.cycle as usize] =
                self.calculate_pixel(bus, cartridge);
        }

        self.cycle += 1;

        if self.cycle == CYCLES_PER_SCANLINE {
            self.scanline += 1;
            self.cycle = 0;
        }

        if self.scanline == SCANLINES_PER_FRAME {
            self.scanline = 0;
            self.odd_frame = !self.odd_frame;
            return true;
        }

        false
    }
}
