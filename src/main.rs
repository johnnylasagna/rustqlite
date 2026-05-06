use std::env;

mod repl;
mod statement;
mod storage;

// Import the items we need into scope
use repl::{InputBuffer, execute_meta_command, parse_meta_command, print_prompt};
use statement::{execute_statement, prepare_statement};
use storage::Table;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Must supply database file name as argument");
        std::process::exit(1);
    }

    let filename = &args[1];

    let mut table = Table::new(filename);
    let mut input_buffer = InputBuffer::default();

    loop {
        print_prompt();
        input_buffer.read_input();
        let input = input_buffer.get_buffer();

        if input.starts_with('.') {
            match parse_meta_command(input) {
                Ok(command) => {
                    execute_meta_command(&command, &mut table);
                    continue;
                }
                Err(_) => {
                    println!("Unrecognized command: {}", input);
                    continue;
                }
            }
        }

        let statement = match prepare_statement(input) {
            Ok(s) => s,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };

        match execute_statement(&statement, &mut table) {
            Ok(_) => println!("Executed."),
            Err(_) => println!("Error: Table full."),
        }
    }
}
