mod gameboy;

fn main() -> Result<(), String> {
    let mut gameboy = gameboy::Gameboy::new()?;
    gameboy.load_rom("./roms/testRom.gb".to_string());
    while gameboy.step() {};
    Ok(())
}