use crate::storage::{EMAIL_SIZE, Row, TABLE_MAX_ROWS, Table, USERNAME_SIZE};

/// Statements
pub enum Statement {
    Insert(Row),
    Select,
}

pub fn prepare_statement(input: &str) -> Result<Statement, &str> {
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
                    let id = match id_str.parse::<u32>() {
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

pub fn execute_statement(
    statement: &Statement,
    table: &mut Table,
) -> Result<&'static str, &'static str> {
    match statement {
        Statement::Insert(_) => execute_insert(statement, table),
        Statement::Select => execute_select(table),
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
