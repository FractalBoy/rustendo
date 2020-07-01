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
