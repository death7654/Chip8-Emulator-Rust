/*
TODO
- Add sound
- Add Input
- Fix OPCODES

*/

use rand::Rng;
use rodio::Source;
use rodio::{source::SineWave, OutputStream, Sink};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{self, event::Event};
use std::time::Duration;
use std::{
    env::{self},
    fs::File,
    io::Read,
};

const MEMORY_SIZE: usize = 4096;
const REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_WIDTH: usize = 64;
const KEYPAD_SIZE: usize = 16;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (DISPLAY_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (DISPLAY_HEIGHT as u32) * SCALE;
const TICKS_PER_FRAME: usize = 10;

//addresses
const START_ADDRESS: u16 = 0x200;

//fonts
const FONTSIZE: usize = 80;
const FONT_SET: [u8; FONTSIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

/*
0x000 - 0x1ff //the interpreter
0x050-0x0a0 //font
0x200-0xfff //program space
 */

pub struct Emulator {
    pc: u16,
    ram: [u8; MEMORY_SIZE],
    ram_index: u16,
    //uses boolean as its just black and white pixels
    display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    stack: [u16; STACK_SIZE],
    registers: [u8; REGISTERS],
    sp: u16,
    //uses boolean to identify keypresses
    keypad: [bool; KEYPAD_SIZE],
    sound_timer: u8,
    delay_timer: u8,
}

impl Emulator {
    pub fn new() -> Self {
        let mut new_emulator = Self {
            pc: START_ADDRESS,
            ram: [0; MEMORY_SIZE],
            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            ram_index: 0,
            stack: [0; STACK_SIZE],
            registers: [0; REGISTERS],
            sp: 0,
            keypad: [false; KEYPAD_SIZE],
            sound_timer: 0,
            delay_timer: 0,
        };
        new_emulator.ram[..FONTSIZE].copy_from_slice(&FONT_SET);
        new_emulator
    }
    pub fn reset(&mut self) {
        self.pc = START_ADDRESS;
        self.ram = [0; MEMORY_SIZE];
        self.display = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        self.ram_index = 0;
        self.stack = [0; STACK_SIZE];
        self.registers = [0; REGISTERS];
        self.sp = 0;
        self.keypad = [false; KEYPAD_SIZE];
        self.sound_timer = 0;
        self.delay_timer = 0;
        self.ram[..FONTSIZE].copy_from_slice(&FONT_SET);
    }
    pub fn cycle(&mut self) {
        let op = self.fetch();

        self.decode(op);
    }
    fn timer(&mut self) -> bool {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            return true;
        }
        return false;
    }
    //decrements pc
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
    fn load(&mut self, data: &[u8]) {
        let start_address = START_ADDRESS as usize;
        let end_address = &start_address + data.len();
        self.ram[start_address..end_address].copy_from_slice(data);
    }
    //increments pc
    fn push(&mut self, value: u16) {
        self.stack[self.sp as usize] = value;
        self.sp += 1;
    }
    pub fn get_display(&self) -> &[bool] {
        &self.display
    }
    pub fn keypress(&mut self, key: usize, pressed: bool) {
        self.keypad[key] = pressed;
    }

    fn fetch(&mut self) -> u16 {
        let upper_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (upper_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }
    fn decode(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return,
            (0, 0, 0xE, 0) => self.display = [false; DISPLAY_HEIGHT * DISPLAY_WIDTH],
            (0, 0, 0xE, 0xE) => {
                let return_address = self.pop();
                self.pc = return_address;
            }
            (1, _, _, _) => {
                let address = op & 0xFFF;
                self.pc = address;
            }
            (2, _, _, _) => {
                let address = op & 0xFFF;
                self.push(self.pc);
                self.pc = address;
            }
            (3, _, _, _) => {
                let vx = digit2 as usize;
                let byte = (op & 0xFF) as u8;
                if self.registers[vx] == byte {
                    self.pc += 2;
                }
            }
            (4, _, _, _) => {
                let vx = digit2 as usize;
                let byte = (op & 0xFF) as u8;
                if self.registers[vx] != byte {
                    self.pc += 2;
                }
            }
            (5, _, _, 0) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                if self.registers[vx] == self.registers[vy] {
                    self.pc += 2;
                }
            }
            (6, _, _, _) => {
                let vx = digit2 as usize;
                let byte = (op & 0xFF) as u8;
                self.registers[vx] = byte;
            }
            (7, _, _, _) => {
                let vx = digit2 as usize;
                let byte = (op & 0xFF) as u8;
                self.registers[vx] = self.registers[vx].wrapping_add(byte);
            }
            (8, _, _, 0) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                self.registers[vx] = self.registers[vy];
            }
            (8, _, _, 1) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                self.registers[vx] |= self.registers[vy];
            }
            (8, _, _, 2) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                self.registers[vx] &= self.registers[vy];
            }
            (8, _, _, 3) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                self.registers[vx] ^= self.registers[vy];
            }
            (8, _, _, 4) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                let (new_vx, carry) = self.registers[vx].overflowing_add(self.registers[vy]);
                let new_vf = if carry { 1 } else { 0 };

                self.registers[vx] = new_vx;
                self.registers[0xF] = new_vf;
            }
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.registers[x].overflowing_sub(self.registers[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.registers[x] = new_vx;
                self.registers[0xF] = new_vf;
            }
            (8, _, _, 6) => {
                let vx = digit2 as usize;
                self.registers[vx] >>= 1;
                self.registers[0xF] = self.registers[vx] & 1;
            }
            (8, _, _, 7) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                let (new_vx, borrow) = self.registers[vy].overflowing_sub(self.registers[vx]);
                let new_vf = if borrow { 0 } else { 1 };

                self.registers[vx] = new_vx;
                self.registers[0xF] = new_vf;
            }
            (8, _, _, 0xE) => {
                let vx = digit2 as usize;
                self.registers[vx] <<= 1;
                self.registers[0xF] = (self.registers[vx] >> 7) & 1;
            }
            (9, _, _, 0) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                if self.registers[vx] != self.registers[vy] {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => {
                let address = op & 0xFFF;
                self.ram_index = address
            }
            (0xB, _, _, _) => {
                let address = op & 0xFFF;
                self.pc = (self.registers[0] as u16) + address;
            }
            (0xC, _, _, _) => {
                let vx = digit2 as usize;
                let address = (op & 0xFF) as u8;
                let rng: u8 = rand::thread_rng().gen();
                self.registers[vx] = rng & address;
            }
            (0xD, _, _, _) => {
                let x_coordinate = self.registers[digit2 as usize] as u16;
                let y_coordinate = self.registers[digit3 as usize] as u16;
                let rows = digit4;

                let mut flipped = false;

                for y_axis in 0..rows {
                    let address = self.ram_index + y_axis as u16;
                    let pixels = self.ram[address as usize];
                    //8 as there are 8 pixels and 64/8 = 8
                    for x_axis in 0..8 {
                        if (pixels & (0b1000_0000 >> x_axis)) != 0 {
                            //as the chip8 interperter wraps around the screen the modulus operator will wrap it for us

                            let x = (x_coordinate + x_axis) as usize % DISPLAY_WIDTH;
                            let y = (y_coordinate + y_axis) as usize % DISPLAY_HEIGHT;

                            // Get our pixel's index for our 1D screen array
                            let idx = x + DISPLAY_WIDTH * y;
                            // Check if we're about to flip the pixel and set
                            flipped |= self.display[idx];
                            self.display[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.registers[0xF] = 1
                } else {
                    self.registers[0xF] = 0
                }
            }
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.registers[x];
                let key_pressed = self.keypad[vx as usize];
                if key_pressed {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.registers[x];
                let key_pressed = self.keypad[vx as usize];
                if !key_pressed {
                    self.pc += 2;
                }
            }
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.registers[x] = self.delay_timer;
            }
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keypad.len() {
                    if self.keypad[i] {
                        self.registers[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            }
            (0xF, _, 1, 5) => {
                let vx = digit2 as usize;
                self.delay_timer = self.registers[vx];
            }
            (0xF, _, 1, 8) => {
                let vx = digit2 as usize;
                self.sound_timer = self.registers[vx];
            }
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.registers[x] as u16;
                self.ram_index = self.ram_index.wrapping_add(vx);
            }
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.registers[x] as u16;
                self.ram_index = c * 5;
            }
            (0xF, _, 3, 3) => {
                //todo make faster
                let x = digit2 as usize;
                let vx = self.registers[x] as f32;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.ram_index as usize] = hundreds;
                self.ram[(self.ram_index + 1) as usize] = tens;
                self.ram[(self.ram_index + 2) as usize] = ones;
            }
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.ram_index as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.registers[idx];
                }
            }
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.ram_index as usize;
                for idx in 0..=x {
                    self.registers[idx] = self.ram[i + idx];
                }
            }
            (_, _, _, _) => {
                println!("Unimplemnted OP code {:#04x}", op)
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
    let args: Vec<_> = env::args().collect();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    //opengl generates sprites faster than vulkan
    let window = video_subsystem
        .window("Chip_8", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    //create a canvas with vsync
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    //create new emulator
    let mut chip8 = Emulator::new();
    //read to rom
    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();

    //read file into buffer
    rom.read_to_end(&mut buffer).unwrap();

    //load rom
    chip8.load(&buffer);

    //sound
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let source = SineWave::new(440.0)
        .take_duration(Duration::from_secs_f32(5.0))
        .amplify(0.20)
        .repeat_infinite();
    sink.append(source);
    sink.pause();
    let mut paused = true;
    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit { .. } => {
                    break 'gameloop;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    let key = get_input(key);
                    match key {
                        Some(key) => {
                            chip8.keypress(key, true);
                        }
                        None => {
                            println!("Invalid Input")
                        }
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    let key = get_input(key);
                    match key {
                        Some(key) => {
                            chip8.keypress(key, false);
                        }
                        None => {}
                    }
                }
                _ => (),
            }
        }
        let a = chip8.timer();
        if a == true {
            if paused {
                sink.play();
                paused = false;
            }
        } else {
            if !paused {
                sink.pause();
                paused = true;
            }
        }
        for _ in 0..TICKS_PER_FRAME {
            chip8.cycle();
        }
        //tick both values
        let _ = chip8.timer();
        draw_screen(&chip8, &mut canvas);
    }
}

fn draw_screen(emu: &Emulator, canvas: &mut Canvas<Window>) {
    // Clear canvas as black
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = emu.get_display();
    // Now set draw color to white, iterate through each point and see if it should be drawn
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            let x = (i % DISPLAY_WIDTH) as u32;
            let y = (i / DISPLAY_WIDTH) as u32;

            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}

fn get_input(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
