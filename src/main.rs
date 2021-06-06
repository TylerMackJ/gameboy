mod gameboy;

fn main() -> Result<(), String> {
    let mut gameboy = gameboy::Gameboy::new()?;
    match gameboy.load_rom("./roms/testRom.gb".to_string()) {
        Ok(_t) => {},
        Err(_t) => return Err("Error loading rom".to_string()),
    }
    while gameboy.step()? {};
    Ok(())
}