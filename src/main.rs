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
    sp: u16,
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
            sp: 0,
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
    //decrements pc
    fn pop(&mut self)-> u16{
        self.sp -=1;
        self.stack[self.sp as usize]
    }
    //increments pc
    fn push(&mut self, value: u16)
    {
        self.stack[self.sp as usize] = value;
        self.sp +=1;
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
        let digit1 = (op & 0xf000) >> 12;
        let digit2 = (op & 0x0f00) >> 8;
        let digit3 = (op & 0x00f0) >> 4;
        let digit4 = (op & 0x000f);

        match (digit1, digit2, digit3, digit4)
        {
            (0,0,0,0) => {return}
            (0,0,0xe, 0) => {self.display = [false;DISPLAY_HEIGHT*DISPLAY_WIDTH]}
            (0,0,0xe, 0xe) => {
                let return_address = self.pop();
            self.pc = return_address;}
            (1,_,_,_) => {
                let address = op & 0x0fff;
                self.pc = address;
            }
            (2,_,_,_) => {
                let address = op& 0x0fff;
                self.push(self.pc);
                self.pc = address;
            }
            (3,_,_,_) => {
                //todo check for better methods
                let vx = (op & 0x00ff) >> 8;
                let byte = op & 0x00ff;
                if vx == byte{
                    self.pc +=2;
                }

            }
            (4,_,_,_) => {
                let vx = (op & 0x0f00)>>8;
                let byte = op & 0x00ff;
                if vx != byte
                {
                    self.pc +=2;
                }
            }
            (5,_,_,_) => {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                if self.registers[vx as usize] == self.registers[vy as usize]
                {
                    self.pc +=2;
                }
            }
            (6,_,_,_) => {
                let vx = (op & 0xf00) >> 8;
                let byte =  (op & 0x00ff) as u8;
                self.registers[vx as usize] = byte;
            }
            (7,_,_,_) => {
                let vx = (op & 0xf00) >> 8;
                let byte = (op & 0x00f0) as u8;
                self.registers[vx as usize] += byte;
            }
            (8,_,_,0) => {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                self.registers[vx as usize] = self.registers[vy as usize];
            }
            (8,_,_,1) => 
            {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                self.registers[vx as usize] |= self.registers[vy as usize];
            }
            (8,0,0,2) =>
            {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                self.registers[vx as usize] &= self.registers[vy as usize];
            }
            (8,_,_,3) =>
            {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                self.registers[vx as usize] ^= self.registers[vy as usize];
            }
            (8,_,_,4) =>
            {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                let (new_vx, carry) = self.registers[vx as usize].overflowing_add(self.registers[vy as usize]);
                if carry
                {
                    self.registers[0xf] = 1;
                }
                else {
                    self.registers[0xf] = 0;
                }

                self.registers[vx as usize] = new_vx;

            }
            (8,_,_,5) => 
            {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                if self.registers[vx as usize] > self.registers[vy as usize]
                {
                    self.registers[0xf] = 1;
                }
                else {
                    self.registers[0xf] = 0;
                }
                self.registers[vx as usize] -= self.registers[vy as usize];
            }
            (8,_,_,6) =>
            {
                let vx = (op & 0x0f00) >> 8;
                self.registers[0xf] = self.registers[vx as usize] & 0x1;
                self.registers[vx as usize] >>=1;
            }
            (8,_,_,7) => 
            {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                if self.registers[vx as usize] > self.registers[vy as usize]
                {
                    self.registers[0xf] = 1;
                }
                else {
                    self.registers[0xf] = 0;
                }
                self.registers[vx as usize] -= self.registers[vy as usize];
            }
            (8,_,_,0xE) =>
            {
                let vx = (op & 0x0f00) >> 8;
                self.registers[0xf] = (self.registers[vx as usize] & 0x1) >> 7;
                self.registers[vx as usize] <<=1;
            }
            (9,_,_,0) =>
            {
                let vx = (op & 0x0f00) >> 8;
                let vy = (op & 0x00f0) >> 4;
                if self.registers[vx as usize] != self.registers[vy as usize]
                {
                    self.pc +=2;
                }
            }
            (0xA,_,_,_) =>
            {
                let address = op & 0x0fff;
                self.ram_index = address
            }
            (_,_,_,_)=> {println!("Unimplemnted OP code")}
        }

    }

}

fn main() {
    println!("Hello, world!");
}
