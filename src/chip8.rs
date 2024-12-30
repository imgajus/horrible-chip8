use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use minifb::{Window, WindowOptions, Key, Scale};
use rand::Rng;
use super::constants::*;

pub struct Sys {
    ram : [u8; RAM_SIZE],
    index_register: u16,
    program_counter: u16,
    delay_timer: u8,
    sound_timer: u8,
    v_register: [u8; 16],
    pub(crate) buffer: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    stack: Vec<u16>,
    cycle_speed: f64,
    key_map: HashMap<u8, Key>
}

impl Sys {
    pub fn new() -> Sys {
        return Sys{
            ram : [0; RAM_SIZE],
            index_register: 0,
            program_counter: 0,
            delay_timer: 0,
            sound_timer: 0,
            v_register: [0; 16],
            buffer: [false; DISPLAY_WIDTH*DISPLAY_HEIGHT],
            stack: Vec::new(),
            cycle_speed: 60.0/SPEED,
            key_map: HashMap::from([
                (0x1, Key::Key1),
                (0x2, Key::Key2),
                (0x3, Key::Key3),
                (0xC, Key::Key4),
                (0x4, Key::Q),
                (0x5, Key::W),
                (0x6, Key::E),
                (0xD, Key::R),
                (0x7, Key::A),
                (0x8, Key::S),
                (0x9, Key::D),
                (0xE, Key::F),
                (0xA, Key::Y),
                (0x0, Key::X),
                (0xB, Key::C),
                (0xF, Key::V),
            ])
        }
    }
    pub fn initialize(&mut self) {
        self.load_into_ram(FONTSET_SIZE, &FONT)
    }
    pub fn pop(&mut self) -> Option<u16> {
        self.stack.pop()
    }
    pub fn push(&mut self, action: u16) {
        self.stack.push(action)
    }
    pub fn load_program(&mut self, filepath: String) -> Result<(), std::io::Error>{
        let program = fs::read(&filepath)?;
        self.load_into_ram(0x200, program.as_slice());
        self.program_counter = 0x200;
        Ok(())
    }
    fn load_into_ram(&mut self, position: usize, data: &[u8]) {
        self.ram[position..position + data.len()].copy_from_slice(data);
    }
    pub fn _combine_bytes(data: [u8; 2]) -> u16 {
        ((data[0] as u16) << 8) | (data[1] as u16)
    }
    pub fn fetch(&mut self) -> u16 {

        let data: u16 = Sys::_combine_bytes(
            [self.ram[self.program_counter as usize], self.ram[(self.program_counter+1) as usize]]
        );

        self.program_counter += 2;
        data
    }
    pub fn decode_execute(&mut self, data: u16, mut window: &mut Window) {
        let op: u16 = (data & 0xF000); //first nibble
        let x: u16 = (data & 0xF00) >> 8; //second nibble
        let y: u16 = (data & 0xF0) >> 4; //third nibble
        let n: u16 = data & 0xF; //fourth nibble
        let nn: u16 = data & 0xFF; //third & fourth nibbles
        let nnn: u16 = data & 0xFFF; //last three nibbles
        println!(
            "\nInstruction Breakdown:\n\
             Full:  {:#06x} ({})\n\
             op:    {:#05x} ({})\n\
             x:     {:#04x} ({})\n\
             y:     {:#04x} ({})\n\
             n:     {:#03x} ({})\n\
             nn:    {:#04x} ({})\n\
             nnn:   {:#05x} ({})\n\
             pc:    {:#05x} ({})",
            data, data,
            op, op,
            x, x,
            y, y,
            n, n,
            nn, nn,
            nnn, nnn,
            self.program_counter, self.program_counter
        );
        fn panic_unexpected(instruction: &u16, context: &u16) {
            panic!("Unexpected instruction '{:#06x}': {}", instruction, context);
        }
        match op {
            0x0000 => match nnn {
                0x00E0 => {
                    self.buffer.fill(false);
                    let screen = self.translate_buffer(self.buffer);
                    window.update_with_buffer(&screen, 64, 32).unwrap()
                },
                0x00EE => {
                    self.program_counter = self.stack.pop().expect("STACK IS EMPTY");
                }
                _ => panic_unexpected(&op, &nnn)
            },
            0x1000 => self.program_counter = nnn,
            0x2000 => {
                self.stack.push(self.program_counter);
                self.program_counter = nnn;
            },
            0x3000 => {
                if self.v_register[x as usize] == nn as u8 { self.program_counter += 2 }
            },
            0x4000 => { if self.v_register[x as usize] != nn as u8 { self.program_counter += 2 }},
            0x5000 => { if self.v_register[x as usize] == self.v_register[y as usize] {
                self.program_counter += 2 }},
            0x6000 => {
                self.v_register[x as usize] = nn as u8;
                },
            0x7000 => {
                let add: (u8, bool) = self.v_register[x as usize].overflowing_add(nn as u8);
                self.v_register[x as usize] = add.0
                },
            0x8000 => match n {
                0 => self.v_register[x as usize] = self.v_register[y as usize],
                1 => self.v_register[x as usize] |= self.v_register[y as usize],
                2 => self.v_register[x as usize] &= self.v_register[y as usize],
                3 => self.v_register[x as usize] ^= self.v_register[y as usize],
                4 => {
                    let add: (u8, bool) = self.v_register[x as usize].overflowing_add(self.v_register[y as usize]);
                    if add.1 { self.v_register[0xF] = 1} else { self.v_register[0xF] = 0}
                    self.v_register[x as usize] = add.0;
                },
                5 => {
                    let sub: (u8, bool) = self.v_register[x as usize].overflowing_sub(self.v_register[y as usize]);
                    if sub.1 { self.v_register[0xF] = 0} else {self.v_register[0xF] = 1}
                    self.v_register[x as usize] = sub.0;
                }
                6 => {
                    self.v_register[x as usize] = self.v_register[y as usize];
                    self.v_register[x as usize] >>= 1;
                }
                7 => {
                    let sub: (u8, bool) = self.v_register[y as usize].overflowing_sub(self.v_register[x as usize]);
                    if sub.1 { self.v_register[0xF] = 0} else {self.v_register[0xF] = 1}
                    self.v_register[x as usize] = sub.0;
                },
                0xA => self.index_register = nnn,
                0xE => {
                    self.v_register[x as usize] = self.v_register[y as usize];
                    self.v_register[x as usize] <<= 1;
                }
                _ => panic_unexpected(&op, &n)
            }
            0x9000 => { if self.v_register[x as usize] != self.v_register[y as usize] {
                self.program_counter += 2 }},
            0xA000 => self.index_register = nnn,
            0xB000 => self.program_counter = nnn.overflowing_add(self.v_register[0] as u16).0, //not sure
            0xC000 => {
                let mut rng = rand::thread_rng();
                self.v_register[x as usize] = (nn & rng.gen::<u16>()) as u8
            }
            0xD000 => {
                let x_pos = self.v_register[x as usize] % 64;;
                let mut y_coord = self.v_register[y as usize] % 32;
                self.v_register[0xF] = 0;
                for i in 0..n {
                    let mut x_coord = x_pos;  // Reset X to original position for each row
                    let mut mask: u8 = 0b10000000;
                    let sprite: u8 = self.ram[(self.index_register + i) as usize];
                    for a in 0..8 {
                        println!("Current Sprite {:#04b} Current x_coord {} Current y_coord {}", sprite, x_coord, y_coord);
                        if sprite & mask != 0 {
                            let index_coord: usize = ((y_coord as usize) * 64 + (x_coord as usize)) as usize;
                            match self.buffer[index_coord] {
                                true => {
                                    self.buffer[index_coord] = false;
                                    self.v_register[0xF] = 1;
                                }
                                false => {
                                    self.buffer[index_coord] = true;
                                }
                            }
                        }
                        mask >>= 1;
                        x_coord += 1;
                        if x_coord >= 64 { break; }
                    }
                    if y_coord >= 32 { break; }
                    y_coord += 1;
                }
                let screen = self.translate_buffer(self.buffer);
                window.update_with_buffer(&screen, 64, 32).unwrap()

            },
            0xE000 => match nn {
                0x9E => println!("Input, TODO"),
                _ => println!("Input, unknown")
            }
            _ => panic_unexpected(&op, &nnn)
        }
    }
    pub fn get_cycle_speed(&self) -> u64 {
        (self.cycle_speed as u64) * 1000000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_bytes() {
        let sys = Sys::new();
        assert_eq!(Sys::_combine_bytes([0x12, 0x34]), 0x1234);
        assert_eq!(Sys::_combine_bytes([0xAB, 0xCD]), 0xABCD);
        assert_eq!(Sys::_combine_bytes([0x00, 0xFF]), 0x00FF);
        assert_eq!(Sys::_combine_bytes([0xFF, 0x00]), 0xFF00);
    }
    #[test]
    fn test_load_into_ram() {
        let mut sys = Sys::new();
        let test_data = [0x1, 0x2, 0x3, 0x4];
        sys.load_into_ram(100, &test_data);
        assert_eq!(&sys.ram[100..104], &test_data);
    }
    #[test]
    fn test_stack_operations() {
        let mut sys = Sys::new();
        sys.push(0x123);
        assert_eq!(sys.pop(), Some(0x123));
        assert_eq!(sys.pop(), None);  // Empty stack
    }
    #[test]
    fn test_system_init() {
        let sys = Sys::new();
        assert_eq!(sys.program_counter, 0);
        assert_eq!(sys.index_register, 0);
        assert_eq!(sys.v_register, [0; 16]);
        assert!(sys.stack.is_empty());
    }
    #[test]
    fn test_font_loading() {
        let mut sys = Sys::new();
        sys.load_into_ram(80, &FONT);
        assert_eq!(&sys.ram[80..160], &FONT);
    }
    #[test]
    fn test_fetch() {
        let mut sys = Sys::new();
        sys.ram[0] = 0x12;
        sys.ram[1] = 0x34;
        assert_eq!(sys.fetch(), 0x1234);
        assert_eq!(sys.program_counter, 2);
    }

    #[test]
    fn test_fetch_from_different_positions() {
        let mut sys = Sys::new();
        sys.program_counter = 100;  // Start from different position
        sys.ram[100] = 0xAB;
        sys.ram[101] = 0xCD;
        assert_eq!(sys.fetch(), 0xABCD);
        assert_eq!(sys.program_counter, 102);
    }

    #[test]
    fn test_multiple_fetches() {
        let mut sys = Sys::new();
        sys.ram[0] = 0x12;
        sys.ram[1] = 0x34;
        sys.ram[2] = 0x56;
        sys.ram[3] = 0x78;

        assert_eq!(sys.fetch(), 0x1234);
        assert_eq!(sys.fetch(), 0x5678);
        assert_eq!(sys.program_counter, 4);
    }
}