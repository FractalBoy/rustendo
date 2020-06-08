pub trait Connect {
    fn read(&self, address_high: u8, address_low: u8) -> Option<u8>;
    fn write(&mut self, address_high: u8, address_low: u8, data: u8);
    fn clock(&mut self);
}

pub struct Bus {
    adh: u8,
    adl: u8,
    data: u8,
    connected: Vec<Box<dyn Connect>>,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            adl: 0,
            adh: 0,
            data: 0,
            connected: vec![],
        }
    }

    pub fn connect(&mut self, connect: Box<dyn Connect>) {
        self.connected.push(connect);
    }

    pub fn write_address(&mut self, address_high: u8, address_low: u8) {
        self.adh = address_high;
        self.adl = address_low;
    }

    pub fn write_data(&mut self, data: u8) {
        self.data = data;
    }

    pub fn read(&self) -> u8 {
        for connect in &self.connected {
            if let Some(data) = connect.read(self.adh, self.adl) {
                return data;
            }
        }

        0
    }

    pub fn write(&mut self) {
        for connect in &mut self.connected {
            connect.write(self.adh, self.adl, self.data);
        }
    }

    #[cfg(test)]
    pub fn read_directly(&mut self, address_high: u8, address_low: u8) -> u8 {
        self.adh = address_high;
        self.adl = address_low;
        self.read()
    }

    pub fn clock(&mut self) {
        for connect in &mut self.connected {
            connect.clock();
        }
    }
}
