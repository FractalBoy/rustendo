use std::cell::RefCell;
use std::rc::Rc;

pub trait Connect {
    fn read(&self, address_high: u8, address_low: u8) -> Option<u8>;
    fn write(&mut self, address_high: u8, address_low: u8, data: u8);
}

pub struct Bus {
    adh: u8,
    adl: u8,
    data: u8,
    connected: Vec<Rc<RefCell<Box<dyn Connect>>>>,
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

    pub fn connect(&mut self, connect: &Rc<RefCell<Box<dyn Connect>>>) {
        self.connected.push(Rc::clone(connect));
    }

    pub fn write_address(&mut self, address_high: u8, address_low: u8) {
        self.adh = address_high;
        self.adl = address_low;
    }

    pub fn read_address(&self) -> (u8, u8) {
        (self.adh, self.adl)
    }

    pub fn write_data(&mut self, data: u8) {
        self.data = data;
    }

    pub fn read(&self) -> u8 {
        for connect in &self.connected {
            if let Some(data) = connect.borrow().read(self.adh, self.adl) {
                return data;
            }
        }

        0
    }

    pub fn write(&self) {
        for connect in &self.connected {
            connect.borrow_mut().write(self.adh, self.adl, self.data);
        }
    }
}
