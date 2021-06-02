mod gameboy;


fn main() -> Result<(), String> {
    let mut window = gameboy::window::SdlWindow::new()?;
    window.event_loop();
    Ok(())
}