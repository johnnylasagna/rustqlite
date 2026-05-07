# rustqlite

`rustqlite` is a small Rust implementation of a toy SQL database built by following the ideas from the [cstack database tutorial](https://cstack.github.io/db_tutorial/).

The project currently implements a simple REPL, basic statement parsing, row serialization into fixed-size pages, and a file-backed pager with a B-tree root node.

## What works today

- Interactive prompt with `.exit` support
- Database filename passed on startup
- `insert <id> <username> <email>` statements
- `select` statements that print all stored rows
- Fixed-size rows with manual serialization and deserialization
- File-backed paging for stored data
- Root-node B-tree initialization and leaf inserts

## Project structure

- `src/main.rs` wires together the REPL, parser, and storage layer
- `src/repl.rs` handles input, the prompt, and meta commands
- `src/statement.rs` parses and executes `insert` and `select`
- `src/storage.rs` defines rows, the pager, and table lifecycle
- `src/btree.rs` defines the root leaf node and insert logic

## Running the project

```bash
cargo run data.db
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

- The first command-line argument must be the database file name.
- Data is written back to the file when you exit with `.exit`.
- Input validation is intentionally minimal and follows the current tutorial progress.

## Reference

- [cstack database tutorial](https://cstack.github.io/db_tutorial/)
