use rustendo_lib::nes::Nes;

fn main() {
    let mut nes = Nes::new();
    loop {
        nes.clock();
    }
}
