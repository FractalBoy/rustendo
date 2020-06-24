#[macro_use]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
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
