use std::{io::{self, Write}, process::Command};

/// Input Buffer
#[derive(Default)]
struct InputBuffer {
    buffer: String,
}

impl InputBuffer {
    fn read_input(&mut self) {
        self.buffer.clear();

        io::stdin()
            .read_line(&mut self.buffer)
            .expect("Failed to Read");
    }

    fn get_buffer(&self) -> &str {
        &self.buffer.trim()
    }
}

// Prompt
fn print_prompt() {
    print!("db > ");
    io::stdout().flush().unwrap();
}

/// Commands
enum MetaCommand {
    Exit,
}

fn parse_meta_command(input: &str) -> Result<MetaCommand, ()> {
    match input {
        ".exit" => Ok(MetaCommand::Exit),
        _ => Err(()),
    }
}

fn execute_meta_command(command: &MetaCommand) {
    match command {
        MetaCommand::Exit => std::process::exit(0)
    }
}

/// Statements
enum Statement {
    Insert,
    Select,
}

fn prepare_statement(input: &str) -> Result<Statement, ()> {
    match input {
        s if s.starts_with("insert") => Ok(Statement::Insert),
        s if s.starts_with("select") => Ok(Statement::Select),
        _ => Err(()),
    }
}

fn execute_statement(statement: &Statement) {
    match statement {
        Statement::Insert => println!("Insert statement executed"),
        Statement::Select => println!("Select statement executed"),
    }
}

fn main() {
    let mut input_buffer = InputBuffer::default();

    loop {
        print_prompt();
        input_buffer.read_input();
        let input = input_buffer.get_buffer();

        if input.starts_with('.') {
            match parse_meta_command(input) {
                Ok(command) => {
                    execute_meta_command(&command);
                    continue;
                }
                Err(_) => {
                    println!("Unrecognized command: {}", input);
                    continue;
                }
            }
        }

        match prepare_statement(input) {
            Ok(statement) => {
                execute_statement(&statement);
                println!("Executed");
            }
            Err(_) => {
                println!("Unrecognized statement at start of: {}", input);
            }
        }
    }
}
