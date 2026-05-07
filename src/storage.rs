use crate::btree::initialize_leaf_node;
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

pub const PAGE_SIZE: usize = 4096;
pub const TABLE_MAX_PAGES: usize = 100;

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
    pub num_pages: usize,
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

        let num_pages = file_size / PAGE_SIZE;

        if file_size % PAGE_SIZE != 0 {
            println!("Db file is not a whole number of pages. Corrupt file");
            std::process::exit(1);
        }

        Pager {
            file,
            file_size,
            num_pages,
            pages: [EMPTY_PAGE; TABLE_MAX_PAGES],
        }
    }

    pub fn get_page(&mut self, page_num: usize) -> Result<&mut [u8], &'static str> {
        if page_num >= TABLE_MAX_PAGES {
            return Err("Tried to access page out of bounds");
        }

        if self.pages[page_num].is_none() {
            let mut page = Box::new([0; PAGE_SIZE]);

            if page_num < self.num_pages {
                self.file
                    .seek(SeekFrom::Start((page_num * PAGE_SIZE) as u64))
                    .map_err(|_| "Failed to seek file")?;
                self.file
                    .read_exact(page.as_mut_slice())
                    .map_err(|_| "Failed to read file")?;
            }

            self.pages[page_num] = Some(page);
        }

        if page_num >= self.num_pages {
            self.num_pages = page_num + 1;
        }

        if let Some(page) = self.pages[page_num].as_mut() {
            Ok(page.as_mut_slice())
        } else {
            Err("Page is empty")
        }
    }

    pub fn flush(&mut self, page_num: usize) -> Result<(), &'static str> {
        if let Some(page) = &self.pages[page_num] {
            self.file
                .seek(SeekFrom::Start((page_num * PAGE_SIZE) as u64))
                .map_err(|_| "Failed to seek file")?;

            self.file
                .write_all(&page[..PAGE_SIZE])
                .map_err(|_| "Failed to write to file")?;

            Ok(())
        } else {
            Err("Tried to flush null page")
        }
    }
}

/// Table
pub struct Table {
    pub root_page_num: usize,
    pub pager: Pager,
}

impl Table {
    pub fn new(filename: &str) -> Table {
        let mut pager = Pager::new(filename);
        let root_page_num = 0;

        if pager.num_pages == 0 {
            let root_node = pager.get_page(0).expect("Failed to init root page");
            initialize_leaf_node(root_node);
        }

        Table {
            root_page_num,
            pager,
        }
    }

    pub fn close(&mut self) -> Result<(), &'static str> {
        for i in 0..self.pager.num_pages {
            if self.pager.pages[i].is_some() {
                self.pager.flush(i)?;
            }
        }

        Ok(())
    }
}
