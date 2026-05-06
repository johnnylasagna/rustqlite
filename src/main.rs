use std::{
    io::{self, Write},
    process::exit,
};

// Input buffer is managed as a struct
struct InputBuffer {
    buffer: String,
}

impl InputBuffer {
    fn new() -> InputBuffer {
        InputBuffer {
            buffer: String::new(),
        }
    }

    // If we don't add clear, the new input will be appended
    // to the old one
    fn read_input(&mut self) {
        self.buffer.clear();

        io::stdin()
            .read_line(&mut self.buffer)
            .expect("Failed to Read");
    }

    // Adding trim removes the newline character
    fn get_buffer(&self) -> &str {
        &self.buffer.trim()
    }
}

// print! does not automatically flush out the stream unlike
// println! so we have to manually flush out the stream
fn print_prompt() {
    print!("db > ");
    io::stdout().flush().unwrap();
}

// We handle meta commands, ie. commands starting with '.'
// and statements in different enums
enum MetaCommandResult {
    MetaCommandSuccess,
    MetaCommandUnrecognizedCommands,
}

enum PrepareResult {
    PrepareSuccess,
    PrepareUnrecognizedStatement,
}

// Enums for statement execution
// Derive copy clone is needed to return statement type
// From Statement struct
#[derive(Copy, Clone)]
enum StatementType {
    StatementInsert,
    StatementSelect,
    StatementNone,
}

struct Statement {
    statement_type: StatementType,
}

impl Statement {
    fn new() -> Statement {
        Statement {
            statement_type: StatementType::StatementNone,
        }
    }
    fn set_statement_type(&mut self, statement_type: StatementType) {
        self.statement_type = statement_type;
    }
    fn get_statement_type(&self) -> StatementType {
        self.statement_type
    }
}

fn do_meta_command(input_buffer: &InputBuffer) -> MetaCommandResult {
    if input_buffer.get_buffer() == ".exit" {
        exit(0);
    } else {
        MetaCommandResult::MetaCommandUnrecognizedCommands
    }
}

fn prepare_statement(input_buffer: &InputBuffer, statement: &mut Statement) -> PrepareResult {
    match input_buffer.get_buffer() {
        s if s.starts_with("insert") => {
            statement.set_statement_type(StatementType::StatementInsert);
            PrepareResult::PrepareSuccess
        }
        s if s.starts_with("select") => {
            statement.set_statement_type(StatementType::StatementSelect);
            PrepareResult::PrepareSuccess
        }
        _ => PrepareResult::PrepareUnrecognizedStatement,
    }
}

fn execute_statement(statement: &Statement) {
    match statement.get_statement_type() {
        StatementType::StatementInsert => {
            println!("Insert statement executed");
        }
        StatementType::StatementSelect => {
            println!("Select statement executed");
        }
        StatementType::StatementNone => {
            println!("No statement executed");
        }
    }
}

fn main() {
    let mut input_buffer = InputBuffer::new();

    loop {
        print_prompt();
        input_buffer.read_input();

        if input_buffer.get_buffer().starts_with('.') {
            match do_meta_command(&input_buffer) {
                MetaCommandResult::MetaCommandSuccess => {
                    continue;
                }
                MetaCommandResult::MetaCommandUnrecognizedCommands => {
                    println!("Unrecognized command: {}", input_buffer.get_buffer())
                }
            }
        }

        let mut statement = Statement::new();

        match prepare_statement(&input_buffer, &mut statement) {
            PrepareResult::PrepareSuccess => {}
            PrepareResult::PrepareUnrecognizedStatement => {
                println!(
                    "Unrecognized statement at start of: {}",
                    input_buffer.get_buffer()
                )
            }
        }

        execute_statement(&statement);
        println!("Executed");
    }
}
