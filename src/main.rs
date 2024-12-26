use std::thread::sleep;
use std::time::{Duration, Instant};
use minifb::Window;
use horrible_chip8::Sys;

fn main() {
    let mut emu = Sys::new();
    emu.initialize();
    emu.load_program("".parse().unwrap()).expect("TODO: panic message");
    let mut window: Window = emu.run_display();
    while window.is_open() {;
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

