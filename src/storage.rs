use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

// Constants
pub const ID_SIZE: usize = 4;
pub const USERNAME_SIZE: usize = 32;
pub const EMAIL_SIZE: usize = 255;
pub const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

const ID_OFFSET: usize = 0;
const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;

const PAGE_SIZE: usize = 4096;
pub const TABLE_MAX_PAGES: usize = 100;
pub const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
pub const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

/// Row
pub struct Row {
    pub id: u32,
    pub username: [u8; 32],
    pub email: [u8; 255],
}

impl Row {
    pub fn new(id: u32, username: &str, email: &str) -> Row {
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

    pub fn serialize(&self, buffer: &mut [u8]) {
        buffer[ID_OFFSET..ID_OFFSET + ID_SIZE].copy_from_slice(&self.id.to_le_bytes());
        buffer[USERNAME_OFFSET..USERNAME_OFFSET + USERNAME_SIZE].copy_from_slice(&self.username);
        buffer[EMAIL_OFFSET..EMAIL_OFFSET + EMAIL_SIZE].copy_from_slice(&self.email);
    }

    pub fn deserialize(buffer: &[u8]) -> Row {
        let mut id = [0u8; ID_SIZE];
        id.copy_from_slice(&buffer[ID_OFFSET..ID_OFFSET + ID_SIZE]);

        let mut username = [0u8; USERNAME_SIZE];
        username.copy_from_slice(&buffer[USERNAME_OFFSET..USERNAME_OFFSET + USERNAME_SIZE]);

        let mut email = [0u8; EMAIL_SIZE];
        email.copy_from_slice(&buffer[EMAIL_OFFSET..EMAIL_OFFSET + EMAIL_SIZE]);

        Row {
            id: u32::from_le_bytes(id),
            username,
            email,
        }
    }
}

/// Pager
pub struct Pager {
    file: File,
    pub file_size: usize,
    pages: [Option<Box<[u8; PAGE_SIZE]>>; TABLE_MAX_PAGES],
}

impl Pager {
    pub fn new(filename: &str) -> Pager {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)
            .expect("Database file could not be opened");

        let file_size = file
            .metadata()
            .expect("File metadata could not be accessed")
            .len() as usize;

        const EMPTY_PAGE: Option<Box<[u8; PAGE_SIZE]>> = None;

        Pager {
            file,
            file_size,
            pages: [EMPTY_PAGE; TABLE_MAX_PAGES],
        }
    }

    pub fn get_page(&mut self, page_num: usize) -> Result<&mut [u8], &'static str> {
        if page_num >= TABLE_MAX_PAGES {
            return Err("Tried to access page out of bounds");
        }

        if self.pages[page_num].is_none() {
            let mut page = Box::new([0; PAGE_SIZE]);

            let mut num_pages = self.file_size / PAGE_SIZE;
            if self.file_size % PAGE_SIZE > 0 {
                num_pages += 1;
            }

            if page_num < num_pages {
                let bytes_to_read = if page_num == num_pages - 1 && self.file_size % PAGE_SIZE > 0 {
                    self.file_size % PAGE_SIZE
                } else {
                    PAGE_SIZE
                };

                self.file
                    .seek(SeekFrom::Start((page_num * PAGE_SIZE) as u64))
                    .map_err(|_| "Failed to seek file")?;
                self.file
                    .read_exact(&mut page[..bytes_to_read])
                    .map_err(|_| "Failed to read file")?;
            }

            self.pages[page_num] = Some(page);
        }

        if let Some(page) = self.pages[page_num].as_mut() {
            Ok(page.as_mut_slice())
        } else {
            Err("Page is empty")
        }
    }

    pub fn flush(&mut self, page_num: usize, size: usize) -> Result<(), &'static str> {
        if let Some(page) = &self.pages[page_num] {
            self.file
                .seek(SeekFrom::Start((page_num * PAGE_SIZE) as u64))
                .map_err(|_| "Failed to seek file")?;

            self.file
                .write_all(&page[..size])
                .map_err(|_| "Failed to write to file")?;

            Ok(())
        } else {
            Err("Tried to flush null page")
        }
    }
}

/// Table
pub struct Table {
    pub num_rows: usize,
    pager: Pager,
}

impl Table {
    pub fn new(filename: &str) -> Table {
        let pager = Pager::new(filename);
        let num_rows = pager.file_size / ROW_SIZE;
        Table { num_rows, pager }
    }

    pub fn close(&mut self) -> Result<(), &'static str> {
        let num_full_pages = self.num_rows / ROWS_PER_PAGE;

        for i in 0..num_full_pages {
            if self.pager.pages[i].is_some() {
                self.pager.flush(i, PAGE_SIZE)?;
            }
        }

        let num_additional_rows = self.num_rows % ROWS_PER_PAGE;
        if num_additional_rows > 0 {
            let page_num = num_full_pages;
            if self.pager.pages[page_num].is_some() {
                self.pager.flush(page_num, num_additional_rows * ROW_SIZE)?;
            }
        }

        Ok(())
    }
}

/// Cursor
pub struct Cursor<'a> {
    table: &'a mut Table,
    pub row_num: usize,
    pub end_of_table: bool,
}

impl<'a> Cursor<'a> {
    pub fn start(table: &'a mut Table) -> Cursor<'a> {
        let end_of_table = table.num_rows == 0;
        Cursor {
            table,
            row_num: 0,
            end_of_table,
        }
    }

    pub fn end(table: &'a mut Table) -> Cursor<'a> {
        let row_num = table.num_rows;
        Cursor {
            table,
            row_num,
            end_of_table: true,
        }
    }

    pub fn value(&mut self) -> Result<&mut [u8], &'static str> {
        let page_num = self.row_num / ROWS_PER_PAGE;
        let row_offset = self.row_num % ROWS_PER_PAGE;
        let byte_offset = row_offset * ROW_SIZE;

        let page = self.table.pager.get_page(page_num)?;
        Ok(&mut page[byte_offset..byte_offset + ROW_SIZE])
    }

    pub fn advance(&mut self) {
        self.row_num += 1;
        if self.row_num >= self.table.num_rows {
            self.end_of_table = true;
        }
    }
}
