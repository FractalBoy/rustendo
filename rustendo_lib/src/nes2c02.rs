pub struct Nes2c02 {
    clocks: u32,
}

impl Nes2c02 {
    pub fn new() -> Self {
        Nes2c02 { clocks: 0 }
    }

    fn read(&self, address: u16) -> Option<u8> {
        unimplemented!();
    }

    fn write(&mut self, address: u16, data: u8) {
        unimplemented!();
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
