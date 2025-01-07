use minifb::{Window, WindowOptions, Scale, Menu, MenuItem};
use super::constants::*;
use crate::Sys;
impl Sys {
    pub fn run_display(&mut self) -> Window {

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
        window
    }
    pub fn translate_buffer(&mut self, buffer: [bool; DISPLAY_WIDTH*DISPLAY_HEIGHT]) -> [u32; DISPLAY_WIDTH*DISPLAY_HEIGHT]{
        let mut screen: [u32; DISPLAY_WIDTH*DISPLAY_HEIGHT] = [BLACK_RGB; DISPLAY_WIDTH*DISPLAY_HEIGHT];
        for (index, pixel) in buffer.iter().enumerate() {
            match pixel {
                true => {screen[index] = WHITE_RGB},

                false => {screen[index] = BLACK_RGB}
            }
        }
        screen
    }
}