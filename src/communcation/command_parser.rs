use core::todo;

use crate::communcation::commands;

#[derive(Default)]
pub struct CommandParser {
    buf: [u8; 32],
    len: usize,
}

impl CommandParser {
    pub const fn new() -> Self {
        Self { 
            buf: [0u8; 32],
            len: 0,
        }
    }

    pub fn push(&mut self, byte: u8) -> Option<Command>{
        if byte == b'\n' || byte == b'\r' {
            if self.len == 0 {
                return None
            }
            
            let command = self.parse();

            self.len = 0;

            return command
        }

        if self.len >= self.buf.len() {
            self.len = 0;
            return None
        }

        self.buf[self.len] = byte;
        self.len += 1;

        None
    }

    fn parse(&self) -> Option<Command>{
        match &self.buf[..self.len] {
            b"led on" => Some(Command::LedOn),
            b"led off" => Some(Command::LedOff),
            b"status" => Some(Command::Status),
            b"bootloader" => Some(Command::Bootloader),
            b"reset" => Some(Command::Reset),
            _ => None,
        }
    }

    pub fn handle_command(source: CommandSource, command: Command) -> Response {

        match source {
            CommandSource::Radio => match command {
                Command::Bootloader => todo!(),
                Command::Reset => todo!(),
                Command::LedOn => todo!(),
                Command::LedOff => todo!(),
                Command::Status => todo!(),
            },
            
            CommandSource::Usb => match command {
                Command::Bootloader => commands::enter_bootloader(),
                Command::Reset => todo!(),
                Command::LedOn => todo!(),
                Command::LedOff => todo!(),
                Command::Status => todo!(),
            },
        }
        
    }

}




#[derive(Debug, Clone, Copy)]
pub enum Command {
    Bootloader,
    Reset,
    LedOn,
    LedOff, 
    Status,
}

pub enum CommandSource {
    Radio,
    Usb,
}

pub enum Response {
    Ok,
    Error,
    Status {
        temperature: i32,
        altitude: i32,
    },
}