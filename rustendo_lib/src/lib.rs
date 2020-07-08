#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        if cfg!(feature = "debug") {
            if cfg!(target_arch = "wasm32") {
                web_sys::console::log_1(&format!( $( $t )* ).into());
            } else {
                print!( $( $t )* );
            }
        }
    }
}

macro_rules! bitfield {
    ($s:ident, $t:ty, $u:ty) => {
        struct $s {
            register: $u
        }

        impl $s {
            pub fn new() -> Self {
                $s { register: 0 }
            }

            #[allow(dead_code)]
            pub fn get_field(&self, bits: $t) -> u8 {
                let mask = bits as $u;
                let shift = mask.trailing_zeros();
                (((self.register & mask) >> shift) & 0xFF) as u8
            } 

            #[allow(dead_code)]
            pub fn set_field(&mut self, bits: $t, data: u8) {
                let mask = bits as $u;
                let shift = mask.trailing_zeros();
                let data = (data as $u) << shift;
                // First clear the bits
                self.register &= !mask;
                // Now set them (or leave them cleared)
                self.register |= data & mask;
            }

            #[allow(dead_code)]
            pub fn get(&self) -> $u {
                self.register
            }

            #[allow(dead_code)]
            pub fn get_mut(&mut self) -> &mut $u {
                &mut self.register
            }
        }
    };
}

mod assembler;
pub mod cartridge;
mod controller;
mod cpu_bus;
mod cpu_ram;
mod mappers;
mod mos6502;
pub mod nes;
mod ppu_bus;
mod ppu_ram;
mod ricoh2c02;
