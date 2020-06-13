pub struct Ricoh2c02 {
    clocks: u32,
    ppu_ctrl: u8,
    ppu_mask: u8,
    ppu_status: u8,
    oam_addr: u8,
    oam_data: u8,
    ppu_scroll: u8,
    ppu_addr: u8,
    ppu_data: u8,
    oam_dma: u8,
}

impl Ricoh2c02 {
    pub fn new() -> Self {
        Ricoh2c02 {
            clocks: 0,
            ppu_ctrl: 0,
            ppu_mask: 0,
            ppu_status: 0,
            oam_addr: 0,
            oam_data: 0,
            ppu_scroll: 0,
            ppu_addr: 0,
            ppu_data: 0,
            oam_dma: 0,
        }
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x2000 => 0,
            0x2001 => 0,
            0x2002 => {
                // Clear bit 7
                self.ppu_status = self.ppu_status & !0x80u8;
                self.ppu_status
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
            0x2000 => self.ppu_ctrl = data,
            0x2001 => self.ppu_mask = data,
            0x2002 => self.ppu_status = data,
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

    pub fn clock(&mut self) {
        // Divide input clock by four.
        if self.clocks % 4 != 0 {
            self.clocks = self.clocks.wrapping_add(1);
            return;
        }

        self.clocks = self.clocks.wrapping_add(1);
    }
}
