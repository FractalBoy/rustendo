use crate::bus::Connect;

pub struct Nes2c02 {
    clocks: u32,
}

impl Nes2c02 {
    pub fn new() -> Self {
        Nes2c02 { clocks: 0 }
    }
}

impl Connect for Nes2c02 {
    fn read(&self, address_high: u8, address_low: u8) -> Option<u8> {
        unimplemented!();
    }

    fn write(&mut self, address_high: u8, address_low: u8, data: u8) {
        unimplemented!();
    }

    fn clock(&mut self) {
        // Divide input clock by four.
        if self.clocks % 4 != 0 {
            self.clocks = self.clocks.wrapping_add(1);
            return;
        }

        self.clocks = self.clocks.wrapping_add(1);
    }
}
