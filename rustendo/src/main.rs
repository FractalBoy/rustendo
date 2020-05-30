use rustendo_lib::mos6502::Mos6502;

fn main() {
    let mut mos6502 = Mos6502::new(None);
    while mos6502.clock() {}
}
