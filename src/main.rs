use std::thread::sleep;
use std::time::{Duration, Instant};
use minifb::Window;
use horrible_chip8::Sys;
use rfd::FileDialog;

fn main() {
    let mut timer = Instant::now();
    let mut emu = Sys::new();
    emu.initialize();
    let mut window: Window = emu.run_display();
    emu.load_program("test_opcode.ch8".parse().unwrap()).expect("TODO: panic message");
    while window.is_open() {
        let idk = window.is_menu_pressed();
        if idk.is_some(){
            let rom_file = FileDialog::new()
                .set_directory("/")
                .pick_file();
            let rom = rom_file.unwrap();
            emu.load_program(rom).expect("TODO: panic message");
        }
        if timer.elapsed().as_micros() > 16666 {
            if emu.delay_timer > 0 {emu.delay_timer -= 1}
            if emu.sound_timer > 0 {
                println!("BEEP");
                emu.sound_timer -= 1;
            }
            timer = Instant::now();
        }
        let now = Instant::now();
        let op = emu.fetch();
        emu.decode_execute(op, &mut window);
        let elapsed: u64 = now.elapsed().as_micros() as u64;
        let micro_cycle_speed = emu.get_cycle_speed();
        window.update();
        if micro_cycle_speed > elapsed { 
            sleep(Duration::from_micros(micro_cycle_speed - elapsed))
        } 
    }
}

