use std::{env::{self, args}, fs::File, io::Read};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use rand::Rng;
use sdl2::{self, event::Event};

const MEMORY_SIZE: usize = 4096;
const REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_WIDTH: usize = 64;
const KEYPAD_SIZE: usize = 16;

//addresses
const START_ADDRESS: u16 = 0x200;
const FONT_START_ADDRESS: u16 = 0x050;

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

const TICKS_PER_FRAME: usize = 10;

/*
0x000 - 0x1ff //the interpreter
0x050-0x0a0 //font
0x200-0xfff //program space
 */

pub struct emulator {
    pc: u16,
    ram: [u8; MEMORY_SIZE],
    ram_index: u16,
    //uses boolean as its just black and white pixels
    display: [bool; DISPLAY_HEIGHT * DISPLAY_WIDTH],
    stack: [u16; STACK_SIZE],
    registers: [u8; REGISTERS],
    sp: u16,
    //uses boolean to identify keypresses
    keypad: [bool; KEYPAD_SIZE],
    sound_timer: u8,
    delay_timer: u8,
}

impl emulator {
    pub fn new() -> Self {
        let mut new_emulator = Self {
            pc: START_ADDRESS,
            ram: [0; MEMORY_SIZE],
            display: [false; DISPLAY_HEIGHT * DISPLAY_WIDTH],
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
        self.display = [false; DISPLAY_HEIGHT * DISPLAY_WIDTH];
        self.ram_index = 0;
        self.stack = [0; STACK_SIZE];
        self.registers = [0; REGISTERS];
        self.keypad = [false; KEYPAD_SIZE];
        self.sound_timer = 0;
        self.delay_timer = 0;
    }
    pub fn cycle(&mut self) {
        let op = self.fetch();

        self.decode(op);
    }
    fn timer(&mut self)
    {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            //TODO implement sound
            self.sound_timer -= 1;
        }
    }
    //decrements pc
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
    fn load(&mut self, data: &[u8])
    {
        let start_address = START_ADDRESS as usize;
        let end_address =  &start_address + data.len();
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
        self.pc += 2;
        ((upper_byte << 8) | lower_byte)
    }
    fn decode(&mut self, op: u16) {
        let digit1 = (op & 0xf000) >> 12;
        let digit2 = (op & 0x0f00) >> 8;
        let digit3 = (op & 0x00f0) >> 4;
        let digit4 = (op & 0x000f);

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return,
            (0, 0, 0xe, 0) => self.display = [false; DISPLAY_HEIGHT * DISPLAY_WIDTH],
            (0, 0, 0xe, 0xe) => {
                let return_address = self.pop();
                self.pc = return_address;
            }
            (1, _, _, _) => {
                let address = op & 0x0fff;
                self.pc = address;
            }
            (2, _, _, _) => {
                let address = op & 0x0fff;
                self.push(self.pc);
                self.pc = address;
            }
            (3, _, _, _) => {
                //todo check for better methods
                let vx = (op & 0x00ff) >> 8;
                let byte = op & 0x00ff;
                if vx == byte {
                    self.pc += 2;
                }
            }
            (4, _, _, _) => {
                let vx = (op & 0x0f00) >> 8;
                let byte = op & 0x00ff;
                if vx != byte {
                    self.pc += 2;
                }
            }
            (5, _, _, _) => {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                if self.registers[vx as usize] == self.registers[vy as usize] {
                    self.pc += 2;
                }
            }
            (6, _, _, _) => {
                let vx = (op & 0xf00) >> 8;
                let byte = (op & 0x00ff) as u8;
                self.registers[vx as usize] = byte;
            }
            (7, _, _, _) => {
                let vx = (op & 0xf00) >> 8;
                let byte = (op & 0x00f0) as u8;
                self.registers[vx as usize] = self.registers[vx as usize].wrapping_add(byte);
            }
            (8, _, _, 0) => {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                self.registers[vx as usize] = self.registers[vy as usize];
            }
            (8, _, _, 1) => {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                self.registers[vx as usize] |= self.registers[vy as usize];
            }
            (8, 0, 0, 2) => {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                self.registers[vx as usize] &= self.registers[vy as usize];
            }
            (8, _, _, 3) => {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                self.registers[vx as usize] ^= self.registers[vy as usize];
            }
            (8, _, _, 4) => {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                let (new_vx, carry) =
                    self.registers[vx as usize].overflowing_add(self.registers[vy as usize]);
                if carry {
                    self.registers[0xf] = 1;
                } else {
                    self.registers[0xf] = 0;
                }

                self.registers[vx as usize] = new_vx;
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
                let vx = (op & 0x0f00) >> 8;
                self.registers[0xf] = self.registers[vx as usize] & 0x1;
                self.registers[vx as usize] >>= 1;
            }
            (8, _, _, 7) => {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                if self.registers[vx as usize] > self.registers[vy as usize] {
                    self.registers[0xf] = 1;
                } else {
                    self.registers[0xf] = 0;
                }
                self.registers[vx as usize] -= self.registers[vy as usize];
            }
            (8, _, _, 0xE) => {
                let vx = (op & 0x0f00) >> 8;
                self.registers[0xf] = (self.registers[vx as usize] & 0x1) >> 7;
                self.registers[vx as usize] <<= 1;
            }
            (9, _, _, 0) => {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                if self.registers[vx as usize] != self.registers[vy as usize] {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => {
                let address = op & 0x0fff;
                self.ram_index = address
            }
            (0xB, _, _, _) => {
                let address = op & 0x0fff;
                self.pc = (self.registers[0] as u16) + address;
            }
            (0xC, _, _, _) => {
                let vx = (op & 0x0f00) >> 8;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = rand::thread_rng().gen();
                self.registers[vx as usize] = rng & nn;
            }
            (0xD, _, _, _) => {
                let x_coordinate = self.registers[digit2 as usize] as u16;
                let y_coordinate = self.registers[digit3 as usize] as u16;
                let rows = digit4;

                let mut flipped = false;

                for y_axis in 0..rows{
                    let address = self.ram_index + y_axis as u16;
                    let pixels = self.ram[address as usize];
                    //8 as there are 8 pixels and 64/8 = 8
                    for x_axis in 0..8
                    {
                        //as the chip8 interperter wraps around the screen the modulus operator will wrap it for us
                        let x = (x_coordinate + x_axis) as usize % DISPLAY_WIDTH;
                        let y = (y_coordinate + y_axis) as usize % DISPLAY_HEIGHT;

                        let idx = x + DISPLAY_WIDTH * y;

                        flipped |= self.display[idx];
                        self.display[idx] ^= true;

                    }
                }

                if flipped 
                {
                    self.registers[0xf] = 1
                }
                else {
                    self.registers[0xf] = 0
                }
            }
            (0xe,_,9,0xe) =>
            {
                let x = digit2 as usize;
                let vx = self.registers[x];
                let key_pressed = self.keypad[vx as usize];
                if key_pressed
                {
                    self.pc +=2;
                }
            }
            (0xe, _, 0xa, 1) =>
            {
                let x = digit2 as usize;
                let vx = self.registers[x];
                let key_pressed = self.keypad[vx as usize];
                if !key_pressed
                {
                    self.pc +=2;
                }
            }
            (0xf,_,0,7) =>
            {
                let x = digit2 as usize;
                self.registers[x]= self.delay_timer;
            }
            (0xf,_,0,0xa) =>
            {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keypad.len()
                {
                    if self.keypad[i]
                    {
                        self.registers[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed
                {
                    self.pc -=2;
                }
            }
            (0xf,_,1,5) =>
            {
                let vx = digit2 as usize;
                self.delay_timer = self.registers[vx];
            }
            (0xf,_,1,8) =>
            {
                let vx = (op & 0xf00) >> 8;
                self.sound_timer = self.registers[vx as usize];
            }
            (0xf,_,1,0xe) =>
            {
                let x = digit2 as usize;
                let vx = self.registers[x] as u16;
                self.ram_index = self.ram_index.wrapping_add(vx);
            }
            (0xf,_,2,9)=>
            {
                let x = digit2 as usize;
                let c = self.registers[x] as u16;
                self.ram_index = c * 5;
            }
            (0xf,_,3,3) =>
            {
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
            (0xf,_,5,5) =>
            {
                let x = digit2 as usize;
                let i = self.registers[x] as usize;
                for idx in 0..=x{
                    self.ram[i+ idx] = self.registers[idx];
                }
            }
            (0xf,0,6,5) =>
            {
                let x = digit2 as usize;
                let i = self.registers[x] as usize;
                for idx in 0..=x
                {
                    self.registers[idx] = self.ram[i+idx];
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
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Chip_8", (DISPLAY_WIDTH as u32) *10, (DISPLAY_HEIGHT as u32)*10).position_centered().vulkan().build().unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();


    //create new emulator
    let mut chip8 = emulator::new();

    let args: Vec<String> = env::args().collect(); 
    let filename = args.get(1).expect("Usage: program <file>");
    //read to rom
    let mut rom = File::open(filename).expect("Unable to open file");
    let mut buffer = Vec::new();

    //read file into buffer
    rom.read_to_end(&mut buffer).unwrap();

    //load rom
    chip8.load(&buffer);

    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit{..} => {
                    break 'gameloop;
                },
                _ => ()
            }
        }
        for _ in 0..TICKS_PER_FRAME {
            chip8.cycle();
        }
        //tick both values
        chip8.timer();
        draw_screen(&chip8, &mut canvas);
    }


}

fn draw_screen(emu: &emulator, canvas: &mut Canvas<Window>) {
    // Clear canvas as black
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = emu.get_display();
    // Now set draw color to white, iterate through each point and see if it should be drawn
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            // Convert our 1D array's index into a 2D (x,y) position
            let x = (i % DISPLAY_WIDTH) as u32;
            let y = (i / DISPLAY_HEIGHT) as u32;

            // Draw a rectangle at (x,y), scaled up by our SCALE value
            let rect = Rect::new((x * 10) as i32, (y * 10) as i32, 10, 10);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}
