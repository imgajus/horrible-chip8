use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use minifb::{Window, Key, KeyRepeat};
use rand::Rng;
use super::constants::*;

pub struct Sys {
    ram : [u8; RAM_SIZE],
    index_register: u16,
    program_counter: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    v_register: [u8; 16],
    pub(crate) buffer: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    stack: Vec<u16>,
    pub cycle_speed: f64,
    hex_to_key_map: HashMap<u8, Key>,
    key_to_hex_map: HashMap<Key, u8>,
    pub key_pressed: Vec<Key>,
    pub display_updated: bool
}

impl Default for Sys {
    fn default() -> Self {
        Self::new()
    }
}

impl Sys {
    pub fn new() -> Sys {
        Sys{
            ram : [0; RAM_SIZE],
            index_register: 0,
            program_counter: 0,
            delay_timer: 0,
            sound_timer: 0,
            v_register: [0; 16],
            buffer: [false; DISPLAY_WIDTH*DISPLAY_HEIGHT],
            stack: Vec::new(),
            cycle_speed: 1.0/SPEED,
            hex_to_key_map: HashMap::from([
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
            ]),
            key_to_hex_map: HashMap::from([
                (Key::Key1, 0x1),
                (Key::Key2, 0x2),
                (Key::Key3, 0x3),
                (Key::Key4, 0xC),
                (Key::Q, 0x4),
                (Key::W, 0x5),
                (Key::E, 0x6),
                (Key::R, 0xD),
                (Key::A, 0x7),
                (Key::S, 0x8),
                (Key::D, 0x9),
                (Key::F, 0xE),
                (Key::Y, 0xA),
                (Key::X, 0x0),
                (Key::C, 0xB),
                (Key::V, 0xF),
            ]),
            key_pressed: Vec::new(),
            display_updated: false
        }
    }
    pub fn initialize(&mut self) {
        self.load_into_ram(0x50, &FONT)
    }
    pub fn pop(&mut self) -> Option<u16> {
        self.stack.pop()
    }
    pub fn push(&mut self, action: u16) {
        self.stack.push(action)
    }
    pub fn load_program(&mut self, filepath: PathBuf) -> Result<(), std::io::Error>{
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
    pub fn decode_execute(&mut self, data: u16, window: &mut Window){  
        let op: u16 = data & 0xF000; //first nibble
        let x: u16 = (data & 0xF00) >> 8; //second nibble
        let y: u16 = (data & 0xF0) >> 4; //third nibble
        let n: u16 = data & 0xF; //fourth nibble
        let nn: u16 = data & 0xFF; //third & fourth nibbles
        let nnn: u16 = data & 0xFFF; //last three nibbles
        /**(
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
        */
        fn panic_unexpected(instruction: &u16, context: &u16) {
            panic!("Unexpected instruction '{:#06x}': {}", instruction, context);
        }
        match op {
            0x0000 => match nnn {
                0x00E0 => {
                    self.buffer.fill(false);
                    self.display_updated = true;
                },
                0x00EE => {
                    self.program_counter = self.stack.pop().expect("STACK IS EMPTY")
                }
                _ => {
                    panic_unexpected(&op, &nnn)
                }
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
                    self.v_register[x as usize] = add.0;
                    if add.1 { self.v_register[0xF] = 1} else { self.v_register[0xF] = 0}
                },
                5 => {
                    let sub: (u8, bool) = self.v_register[x as usize].overflowing_sub(self.v_register[y as usize]);
                    self.v_register[x as usize] = sub.0;
                    if sub.1 { self.v_register[0xF] = 0} else {self.v_register[0xF] = 1}
                },
                6 => {
                    self.v_register[x as usize] = self.v_register[y as usize];
                    let shifted_out = self.v_register[x as usize] & 0b00000001;
                    self.v_register[x as usize] >>= 1;
                    if shifted_out > 1 { panic!("shifted out bit is greater than one: {}" ,shifted_out)}
                    self.v_register[0xF] = shifted_out
                },
                7 => {
                    let sub: (u8, bool) = self.v_register[y as usize].overflowing_sub(self.v_register[x as usize]);
                    self.v_register[x as usize] = sub.0;
                    if sub.1 { self.v_register[0xF] = 0} else {self.v_register[0xF] = 1}
                },
                0xA => self.index_register = nnn,
                0xE => {
                    let shifted_out = (self.v_register[x as usize] & 0b10000000) >> 7;
                    self.v_register[x as usize] = self.v_register[y as usize];
                    self.v_register[x as usize] <<= 1;
                    if shifted_out > 1 { panic!("shifted out bit is greater than one: {}" ,shifted_out)}
                    self.v_register[0xF] = shifted_out
                }
                _ => panic_unexpected(&op, &n)
            }
            0x9000 => { if self.v_register[x as usize] != self.v_register[y as usize] {
                self.program_counter += 2 }},
            0xA000 => self.index_register = nnn,
            0xB000 => self.program_counter = nnn.overflowing_add(self.v_register[0] as u16).0, //not sure
            0xC000 => {
                let mut rng = rand::rng();
                self.v_register[x as usize] = (nn & rng.random::<u16>()) as u8
            }
            0xD000 => {
                let x_pos = self.v_register[x as usize] % 64;
                let mut y_coord = self.v_register[y as usize] % 32;
                self.v_register[0xF] = 0;
                for i in 0..n {
                    let mut x_coord = x_pos;  // Reset X to original position for each row
                    let mut mask: u8 = 0b10000000;
                    let sprite: u8 = self.ram[(self.index_register + i) as usize];
                    for _a in 0..8 {
                        //println!("Current Sprite {:#04b} Current x_coord {} Current y_coord {}", sprite, x_coord, y_coord);
                        if sprite & mask != 0 && x_coord < 64 && y_coord < 32 {
                            let index_coord: usize = (y_coord as usize) * 64 + (x_coord as usize);
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
                self.display_updated = true;
            },
            0xE000 => match nn {
                0x9E => {
                    //println!("Checking for {} Key", self.v_register[x as usize]);
                    if window.is_key_down(self.hex_to_key_map[&self.v_register[x as usize]]) {
                        //println!("Key {:#02x} is down, skipping instruction", self.v_register[x as usize]);
                        self.program_counter += 2;
                    }
                },
                0xA1 => {
                    if !window.is_key_down(self.hex_to_key_map[&self.v_register[x as usize]]) {
                        //println!("Key {:#02x} is NOT down, skipping instruction", self.v_register[x as usize]);
                        self.program_counter += 2;
                    }
                }
                _ => panic_unexpected(&op, &nn)
            },
            0xF000 => match nn{
                0x07 => self.v_register[x as usize] = self.delay_timer,
                0x15 => self.delay_timer = self.v_register[x as usize],
                0x18 => self.sound_timer = self.v_register[x as usize],
                0x1E => {
                    if self.index_register + self.v_register[x as usize] as u16 > 0xFFF {
                        self.v_register[0xF] = 1;
                        self.index_register += self.v_register[x as usize] as u16;
                        self.index_register -= 0xFFF;
                    } else {
                        self.index_register += self.v_register[x as usize] as u16;
                    }
                },
                0x0A => {
                    let keys = window.get_keys_pressed(KeyRepeat::No);
                    if !keys.is_empty() {
                        if let Some(&hex_value) = keys.iter()
                            .find_map(|&key| self.key_to_hex_map.get(&key)) {
                            self.v_register[x as usize] = hex_value;
                        }
                    } else {
                        // If no key is pressed, decrement PC to repeat this instruction
                        self.program_counter -= 2;
                    }
                },
                0x29 => self.index_register = (0x50 + self.v_register[x as usize]) as u16,
                0x33 => {
                    let mut digit: u8 = self.v_register[x as usize];
                    self.ram[self.index_register as usize + 2] = digit % 10;
                    digit /= 10;
                    self.ram[self.index_register as usize + 1] = digit % 10;
                    digit /= 10;
                    self.ram[self.index_register as usize] = digit;
                },
                0x55 => {
                    for i in 0..x+1 {
                        self.ram[(self.index_register + i) as usize] = self.v_register[i as usize]
                    }
                }
                0x65 => {
                    for i in 0..x+1 {
                        self.v_register[i as usize] = self.ram[(self.index_register + i) as usize]
                    }
                }
                _ => panic_unexpected(&op, &nn),
            },
            _ => panic_unexpected(&op, &nnn)
        }
        
    }
    pub fn update_display(&mut self, window: &mut Window, rom_loaded: bool) {
        if !rom_loaded { 
            window.update();
        } else if self.display_updated {
            let screen = self.translate_buffer(self.buffer);
            window.update_with_buffer(&screen, 64, 32).unwrap();
            self.display_updated = false;
        }
    }
}
