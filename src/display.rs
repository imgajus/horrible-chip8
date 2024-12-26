use std::thread::sleep;
use std::time::Duration;
use minifb::{Window, WindowOptions, Key, Scale, Menu, MenuItem};
use super::constants::*;
use crate::Sys;
impl Sys {
    pub fn run_display(&mut self) -> Window {
        const fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
            let (r, g, b) = (r as u32, g as u32, b as u32);
            (r << 16) | (g << 8) | b
        }

        const WHITE_RGB: u32 = from_u8_rgb(255, 255, 255);
        const BLACK_RGB: u32 = from_u8_rgb(0, 0, 0);

        let mut window = Window::new(
            WINDOW_NAME,
            DISPLAY_WIDTH,
            DISPLAY_HEIGHT,
            WindowOptions {
                scale: Scale::X16,
                ..WindowOptions::default()
            }
        ).unwrap();
        
        let mut menu = Menu::new("File").unwrap();
        let load_rom_menu: MenuItem = MenuItem::new("Load ROM", 0);
        menu.add_menu_item(&load_rom_menu);
        window.add_menu(&menu);
        window.set_target_fps(60);
        return window;
    }
    pub fn translate_buffer(&mut self, mut buffer: [bool; DISPLAY_WIDTH*DISPLAY_HEIGHT]) -> [u32; DISPLAY_WIDTH*DISPLAY_HEIGHT]{
        let mut screen: [u32; DISPLAY_WIDTH*DISPLAY_HEIGHT] = [BLACK_RGB; DISPLAY_WIDTH*DISPLAY_HEIGHT];
        for (index, pixel) in buffer.iter().enumerate() {
            match pixel {
                true => {screen[index] = WHITE_RGB;
                //println!("White value received ({})", index);
                    //sleep(Duration::new(1, 0))
                    ;},

                false => {screen[index] = BLACK_RGB;
                //println!("Black value received ({})", index);
                    //sleep(Duration::new(1, 0))
                    ;}
            }
        }
        return screen;
    }
}