use rustendo_lib::mos6502::Mos6502;

fn main() {
    let mut cpu = Mos6502::new();
    cpu.run();
}
