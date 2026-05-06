use crate::storage::Table;
use std::io::{self, Write};

/// Input Buffer
#[derive(Default)]
pub struct InputBuffer {
    buffer: String,
}

impl InputBuffer {
    pub fn read_input(&mut self) {
        self.buffer.clear();

        io::stdin()
            .read_line(&mut self.buffer)
            .expect("Failed to Read");
    }

    pub fn get_buffer(&self) -> &str {
        self.buffer.trim()
    }
}

// Prompt
pub fn print_prompt() {
    print!("db > ");
    io::stdout().flush().unwrap();
}

/// Commands
pub enum MetaCommand {
    Exit,
}

pub fn parse_meta_command(input: &str) -> Result<MetaCommand, ()> {
    match input {
        ".exit" => Ok(MetaCommand::Exit),
        _ => Err(()),
    }
}

pub fn execute_meta_command(command: &MetaCommand, table: &mut Table) {
    match command {
        MetaCommand::Exit => {
            if let Err(e) = table.close() {
                println!("Error closing table: {}", e);
            }
            std::process::exit(0)
        }
    }
}
