# rustqlite

`rustqlite` is a small Rust implementation of a toy sqlite database built by following the ideas from the [cstack database tutorial](https://cstack.github.io/db_tutorial/).

The project currently implements an in-memory table with a simple REPL, basic statement parsing, and row serialization into fixed-size pages.

## What works today

- Interactive prompt with `.exit` support
- `insert <id> <username> <email>` statements
- `select` statements that print all stored rows
- Fixed-size rows with manual serialization and deserialization
- In-memory paging for stored data

## Project structure

- `src/main.rs` wires together the REPL, parser, and storage layer
- `src/repl.rs` handles input, the prompt, and meta commands
- `src/statement.rs` parses and executes `insert` and `select`
- `src/storage.rs` defines rows, pages, and the in-memory table

## Running the project

```bash
cargo run
```

Example session:

```text
db > insert 1 alice alice@example.com
Executed.
db > select
(1, alice, alice@example.com)
db > .exit
```

## Notes

- This is still an in-memory database, so data is not persisted between runs.
- Input validation is intentionally minimal and follows the current tutorial progress.

## Reference

- [cstack database tutorial](https://cstack.github.io/db_tutorial/)
