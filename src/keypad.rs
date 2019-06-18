use crate::cpu::Interrupts;

#[derive(Debug, Clone, Copy)]
pub enum Button
{
    Left, 
    Right, 
    Up, 
    Down,
    A, 
    B, 
    Start, 
    Select
}

pub enum Selected
{
    Button = 0x20,
    Direction = 0x10,
    MltReq = 0x00
}

pub struct Keypad
{
    buttons: u8,
    directions: u8,
    keypad_sel: u8,
    col: Selected
}

impl Keypad
{
    pub fn new() -> Self
    {
        Keypad {
            buttons: 0xF,
            directions: 0xF,
            keypad_sel: 0,
            col: Selected::Direction
        }
    }

    pub fn read_byte(&self, _addr: u16) -> u8
    {
        match self.col
        {
            Selected::Button => self.buttons,
            Selected::Direction => self.directions,
            Selected::MltReq => 0xF - self.keypad_sel
        }
    }

    pub fn write_byte(&mut self, _addr: u16, val: u8)
    {
        match !val & 0x30
        {
            0x20 => self.col = Selected::Button,
            0x10 => self.col = Selected::Direction,
            0x00 => self.col = Selected::MltReq,

            _ => {}
        }
    }

    pub fn key_down(&mut self, key: Button, intf: &mut u8)
    {
        *intf |= Interrupts::Joypad as u8;
        match key
        {
            Button::Left        => self.directions &= 0xD,
            Button::Right       => self.directions &= 0xE,
            Button::Up          => self.directions &= 0xB,
            Button::Down        => self.directions &= 0x7,
            Button::A           => self.buttons &= 0xE,
            Button::B           => self.buttons &= 0xD,
            Button::Start       => self.buttons &= 0x7,
            Button::Select      => self.buttons &= 0xB
        }
    }

    pub fn key_up(&mut self, key: Button)
    {
        match key
        {
            Button::Left        => self.directions |= !0xD,
            Button::Right       => self.directions |= !0xE,
            Button::Up          => self.directions |= !0xB,
            Button::Down        => self.directions |= !0x7,
            Button::A           => self.buttons |= !0xE,
            Button::B           => self.buttons |= !0xD,
            Button::Start       => self.buttons |= !0x7,
            Button::Select      => self.buttons |= !0xB
        }
    }
}