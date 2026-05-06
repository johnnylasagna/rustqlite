use std::{io::{self, Write}, process::exit};

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

fn main() {
    let mut input_buffer = InputBuffer::new();

    loop {
        print_prompt();
        input_buffer.read_input();

        if input_buffer.get_buffer() == ".exit" {
            exit(0);
        } else {
            println!("Unrecognized command: {}", input_buffer.get_buffer());
        }
    }
}
