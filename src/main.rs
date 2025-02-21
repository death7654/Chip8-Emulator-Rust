const MEMORY_SIZE: usize = 4096;
const REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_WIDTH: usize = 64;
const KEYPAD_SIZE: usize = 16;



//addresses
const START_ADDRESS:u16 = 0x200;
const FONT_START_ADDRESS:u16 = 0x050;

//fonts
const FONTSIZE: usize = 80;
const FONT_SET: [u8; FONTSIZE] =
[
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

/*
0x000 - 0x1ff //the interpreter
0x050-0x0a0 //font
0x200-0xfff //program space
 */

pub struct emulator
{
    pc: u16, 
    ram:[u8; MEMORY_SIZE],
    ram_index: u16,
    //uses boolean as its just black and white pixels
    display: [bool;DISPLAY_HEIGHT*DISPLAY_WIDTH],
    stack: [u16; STACK_SIZE],
    registers: [u8; REGISTERS],
    i_register: u16,
    //uses boolean to identify keypresses
    keypad: [bool; KEYPAD_SIZE],
    sound_timer: u8,
    delay_timer:  u8
}

impl emulator
{
    pub fn new() -> Self
    {
        let mut new_emulator = Self{
            pc: START_ADDRESS,
            ram: [0; MEMORY_SIZE],
            display: [false; DISPLAY_HEIGHT*DISPLAY_WIDTH],
            ram_index: 0,
            stack: [0; STACK_SIZE],
            registers: [0; REGISTERS],
            i_register: 0,
            keypad: [false; KEYPAD_SIZE],
            sound_timer: 0,
            delay_timer: 0
        };
        new_emulator.ram[..FONTSIZE].copy_from_slice(&FONT_SET);
        new_emulator

    }
    pub fn reset(&mut self)
    {
        self.pc =  START_ADDRESS;
        self.ram = [0;MEMORY_SIZE];
        self.display = [false;DISPLAY_HEIGHT*DISPLAY_WIDTH];
        self.ram_index = 0;
        self.stack = [0; STACK_SIZE];
        self.registers = [0; REGISTERS];
        self.i_register = 0;
        self.keypad = [false; KEYPAD_SIZE];
        self.sound_timer = 0;
        self.delay_timer = 0;

    }
    pub fn tick(&mut self)
    {
        let op = self.fetch();

        if self.delay_timer > 0
        {
            self.delay_timer -=1;
        }
        if self.sound_timer > 0
        {
            //TODO implement sound
            self.sound_timer -=1;
        }

    }
    fn fetch(&mut self) -> u16
    {
        let upper_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc+1) as usize] as u16;
        self.pc +=2;
        ((upper_byte << 8) | lower_byte)
    }
    fn decode(&mut self, op:u16)
    {
        
    }

}

fn main() {
    println!("Hello, world!");
}
