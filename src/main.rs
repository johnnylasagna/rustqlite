use std::io::{self, Write};

// Constants
const ID_SIZE: usize = 4;
const USERNAME_SIZE: usize = 32;
const EMAIL_SIZE: usize = 255;
const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

const ID_OFFSET: usize = 0;
const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;

const PAGE_SIZE: usize = 4096;
const TABLE_MAX_PAGES: usize = 100;
const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

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

struct Row {
    id: i32,
    username: [u8; 32],
    email: [u8; 255],
}

impl Row {
    fn new(id: i32, username: &str, email: &str) -> Row {
        let mut username_buffer = [0; USERNAME_SIZE];
        username_buffer[..username.len()].copy_from_slice(username.as_bytes());

        let mut email_buffer = [0; EMAIL_SIZE];
        email_buffer[..email.len()].copy_from_slice(email.as_bytes());

        Row {
            id,
            username: username_buffer,
            email: email_buffer,
        }
    }

    fn serialize(&self, buffer: &mut [u8]) {
        buffer[ID_OFFSET..ID_OFFSET + ID_SIZE].copy_from_slice(&self.id.to_le_bytes());
        buffer[USERNAME_OFFSET..USERNAME_OFFSET + USERNAME_SIZE].copy_from_slice(&self.username);
        buffer[EMAIL_OFFSET..EMAIL_OFFSET + EMAIL_SIZE].copy_from_slice(&self.email);
    }

    fn deserialize(buffer: &[u8]) -> Row {
        let mut id = [0u8; ID_SIZE];
        id.copy_from_slice(&buffer[ID_OFFSET..ID_OFFSET + ID_SIZE]);

        let mut username = [0u8; USERNAME_SIZE];
        username.copy_from_slice(&buffer[USERNAME_OFFSET..USERNAME_OFFSET + USERNAME_SIZE]);

        let mut email = [0u8; EMAIL_SIZE];
        email.copy_from_slice(&buffer[EMAIL_OFFSET..EMAIL_OFFSET + EMAIL_SIZE]);

        Row {
            id: i32::from_le_bytes(id),
            username,
            email,
        }
    }
}

struct Table {
    num_rows: usize,
    pages: [Option<Box<[u8; PAGE_SIZE]>>; TABLE_MAX_PAGES],
}

impl Table {
    fn new() -> Table {
        const EMPTY_PAGE: Option<Box<[u8; PAGE_SIZE]>> = None;
        Table {
            num_rows: 0,
            pages: [EMPTY_PAGE; TABLE_MAX_PAGES],
        }
    }
    fn row_slot_write(&mut self, row_num: usize) -> Result<&mut [u8], &str> {
        let page_num = row_num / ROWS_PER_PAGE;

        if self.pages[page_num].is_none() {
            self.pages[page_num] = Some(Box::new([0; PAGE_SIZE]));
        }

        let row_offset = row_num % ROWS_PER_PAGE;
        let byte_offset = row_offset * ROW_SIZE;
        if let Some(page) = self.pages[page_num].as_mut() {
            return Ok(&mut page[byte_offset..byte_offset + ROW_SIZE]);
        } else {
            return Err("Page not allocated");
        }
    }

    fn row_slot_read(&self, row_num: usize) -> Result<&[u8], &str> {
        let page_num = row_num / ROWS_PER_PAGE;
        let row_offset = row_num % ROWS_PER_PAGE;
        let byte_offset = row_offset * ROW_SIZE;

        if let Some(page) = self.pages[page_num].as_ref() {
            return Ok(&page[byte_offset..byte_offset + ROW_SIZE]);
        } else {
            return Err("Page not allocated");
        }
    }
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
        MetaCommand::Exit => std::process::exit(0),
    }
}

/// Statements
enum Statement {
    Insert(Row),
    Select,
}

fn prepare_statement(input: &str) -> Result<Statement, &str> {
    match input {
        s if s.starts_with("insert") => {
            let mut parts = input.split_whitespace();
            match (parts.next(), parts.next(), parts.next(), parts.next()) {
                (Some("insert"), Some(id_str), Some(username), Some(email)) => {
                    if username.len() > USERNAME_SIZE {
                        return Err("Username size exceeds limit");
                    }
                    if email.len() > EMAIL_SIZE {
                        return Err("Email size exceeds limit");
                    }
                    let id = match id_str.parse::<i32>() {
                        Ok(id_int) => id_int,
                        Err(_) => return Err("ID must be an integer"),
                    };
                    let row = Row::new(id, username, email);
                    return Ok(Statement::Insert(row));
                }
                _ => return Err("Syntax error. Could not parse statement."),
            }
        }
        s if s.starts_with("select") => Ok(Statement::Select),
        _ => Err("Unrecognized statement"),
    }
}

fn execute_insert(statement: &Statement, table: &mut Table) -> Result<&'static str, &'static str> {
    if table.num_rows >= TABLE_MAX_ROWS {
        return Err("Table Full");
    }

    if let Statement::Insert(row) = statement {
        let slot = table
            .row_slot_write(table.num_rows)
            .expect("Failed to write to slot");
        row.serialize(slot);
        table.num_rows += 1;
    }

    Ok("Insert statement executed")
}

fn execute_select(table: &Table) -> Result<&'static str, &'static str> {
    for i in 0..table.num_rows {
        let slot = table.row_slot_read(i).expect("Failed to open slot");
        let row = Row::deserialize(slot);

        let username_str = String::from_utf8_lossy(&row.username);
        let email_str = String::from_utf8_lossy(&row.email);

        println!(
            "({}, {}, {})",
            row.id,
            username_str.trim_end_matches('\0'),
            email_str.trim_end_matches('\0')
        );
    }

    Ok("Select statement executed")
}

fn execute_statement(
    statement: &Statement,
    table: &mut Table,
) -> Result<&'static str, &'static str> {
    match statement {
        Statement::Insert(_) => execute_insert(statement, table),
        Statement::Select => execute_select(table),
    }
}

fn main() {
    let mut table = Table::new();
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
