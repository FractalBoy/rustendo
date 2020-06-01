use crate::bus::{Bus, Connect};
use crate::mos6502::Mos6502;
use crate::ram::Ram;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Rp2a03 {
    mos6502: Mos6502,
    bus: Rc<RefCell<Bus>>,
}

impl Rp2a03 {
    /// Create a new RP2A03 microprocessor.
    ///
    /// Optionally, initialize ram with a slice.
    ///
    /// ```
    /// use rustendo_lib::rp2a03::Rp2a03;
    /// // Initialize without memory
    /// let mut rp2a03 = Rp2a03::new(None);
    /// // Initialize with memory
    /// let mut rp2a03 = Rp2a03::new(Some(&[0x0, 0x1, 0x2]));
    /// ```
    pub fn new(mem: Option<&[u8]>) -> Self {
        let mut ram = Box::new(Ram::new());
        let mut bus = Bus::new();

        if let Some(mem) = mem {
            ram.load_mem(&mem);
        }
        let ram = Rc::new(RefCell::new(ram as Box<dyn Connect>));

        bus.connect(&ram);

        let bus = Rc::new(RefCell::new(bus));

        Rp2a03 {
            mos6502: Mos6502::new(&bus),
            bus,
        }
    }

    /// Run the microprocessor for one clock cycle.
    /// Returns whether the current instruction is complete.
    ///
    /// Really, everything happens in the first clock
    /// cycle and the remaining time is spent doing
    /// nothing. The clock cycles are needed to emulate
    /// the microprocessor correctly.
    ///
    /// ```
    /// use rustendo_lib::rp2a03::Rp2a03;
    /// let mut rp2a03 = Rp2a03::new(None);
    /// // Run until one instruction completes.
    /// while rp2a03.clock() {}
    /// ```
    pub fn clock(&mut self) -> bool {
        self.mos6502.clock()
    }

    /// Get memory at a particular address.
    ///
    /// Useful for testing. Resets internal address bus register
    /// to its old state.
    ///
    /// ```
    /// use rustendo_lib::rp2a03::Rp2a03;
    /// let mut rp2a03 = Rp2a03::new(None);
    /// let mem = rp2a03.read_memory_at_address(0);
    /// assert_eq!(mem, 0x0);
    /// ```
    pub fn read_memory_at_address(&mut self, address: u16) -> u8 {
        let old_address = self.bus.borrow().read_address();
        self.bus
            .borrow_mut()
            .write_address(((address & 0xFF00) >> 8) as u8, (address & 0x00FF) as u8);
        let mem = self.bus.borrow().read();
        self.bus
            .borrow_mut()
            .write_address(old_address.0, old_address.1);
        mem
    }
}
