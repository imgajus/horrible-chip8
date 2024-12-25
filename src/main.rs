use std::thread::sleep;
use std::time::Duration;
use minifb::{Window, WindowOptions, Key, Scale};

const RAM_SIZE: usize = 4096;
const FONTSET_SIZE: usize = 80;
const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_WIDTH: usize = 64;
const WINDOW_NAME: &str = "Horrible CHIP-8";

pub struct Sys {
    ram : [u8; RAM_SIZE],
    index_register: u16,
    program_counter: u16,
    delay_timer: u8,
    sound_timer: u8,
    v_register: [u8; 16],
    display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    stack: Vec<u16>,
    speed: u16
}

impl Sys {
    fn pop(&mut self) -> Option<u16> {
        return self.stack.pop();
    }
    fn push(&mut self, action: u16) {
        return self.stack.push(action);
    }
}

fn main() {
    const fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
        let (r, g, b) = (r as u32, g as u32, b as u32);
        (r << 16) | (g << 8) | b
    }
    const WHITE_RGB: u32 = from_u8_rgb(255, 255, 255);
    const BLACK_RGB: u32 = from_u8_rgb(0, 0, 0);
    let mut buffer: Vec<u32> = vec![BLACK_RGB; DISPLAY_WIDTH*DISPLAY_HEIGHT];
    let mut window = Window::new(
        WINDOW_NAME,
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        WindowOptions {
            scale: Scale::X16,
            ..WindowOptions::default()
        }
    ).unwrap();
    window.set_target_fps(60);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT);
    }
}