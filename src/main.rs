mod gameboy;

fn main() {
    let mut gb = gameboy::Gameboy::new();
    gb.load_rom("./roms/testRom.gb".to_string());
    while gb.step() {}
}