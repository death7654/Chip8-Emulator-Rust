const MEMORY_SIZE: usize = 4096;
const REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_WIDTH: usize = 64;
const KEYPAD_SIZE: usize = 16;

pub struct emulator
{
    pc: u16, 
    ram:[u8; MEMORY_SIZE],
    ram_index: u16,
    //uses boolean as its just black and white pixels
    display: [bool;DISPLAY_HEIGHT*DISPLAY_WIDTH],
    stack: [u16; STACK_SIZE],
    registers: [u8; REGISTERS],
    keypad: [bool; KEYPAD_SIZE],
    sound_timer: u8,
    delay_timer:  u8
}

fn main() {
    println!("Hello, world!");
}
