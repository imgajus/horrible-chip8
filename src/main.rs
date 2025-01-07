use std::thread::sleep;
use std::time::{Duration, Instant};
use minifb::{Window};
use horrible_chip8::Sys;
use rfd::FileDialog;

fn main() {
    let timer_duration = Duration::from_millis(1000 / 60); // 60 Hz
    let mut rom_loaded = false;
    let mut timer = Instant::now();
    let mut emu = Sys::new();
    emu.initialize();
    let mut window: Window = emu.run_display();

    let cycle_duration = Duration::from_secs_f64(emu.cycle_speed);
    let mut last_cycle_time = Instant::now();

    while window.is_open() {
        if window.is_menu_pressed().is_some(){
            let rom_file = FileDialog::new()
                .pick_file();
            let rom = rom_file.unwrap();
            emu.load_program(rom).expect("ROM Could not be loaded");
            rom_loaded = true
        }

        // 60Hz updates
        if timer.elapsed() >= timer_duration {
            window.update();
            if emu.delay_timer > 0 {
                emu.delay_timer -= 1;
            }
            if emu.sound_timer > 0 {
                //println!("BEEP");
                emu.sound_timer -= 1;
            }
            emu.update_display(&mut window, rom_loaded);
            timer = Instant::now();
        }
        if rom_loaded {
            let op = emu.fetch();
            emu.decode_execute(op, &mut window);
        }

        let elapsed = last_cycle_time.elapsed();
        //println!("Instruction: {:#06x}, elapsed: {}, goal cycle duration: {}", op, elapsed.as_micros(), cycle_duration.as_micros());
        if elapsed < cycle_duration {
            //println!("DURATION SMALLER, SLEEPING FOR {}", (cycle_duration - elapsed).as_micros());
            sleep(cycle_duration - elapsed);
        }
        last_cycle_time = Instant::now();
    }
}

