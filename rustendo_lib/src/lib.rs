#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        if cfg!(target_arch = "wasm32") { 
            web_sys::console::log_1(&format!( $( $t )* ).into());
        } else {
            print!( $( $t )* );
        }
    }
}

mod assembler;
mod cpu_bus;
mod ppu_bus;
mod mos6502;
mod cpu_ram;
mod ppu_ram;
pub mod cartridge;
mod ricoh2c02;
mod mappers;
pub mod nes;
mod controller;
