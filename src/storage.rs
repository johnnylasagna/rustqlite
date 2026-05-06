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
    pub id: i32,
    pub username: [u8; 32],
    pub email: [u8; 255],
}

impl Row {
    pub fn new(id: i32, username: &str, email: &str) -> Row {
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
            id: i32::from_le_bytes(id),
            username,
            email,
        }
    }
}

/// Table
pub struct Table {
    pub num_rows: usize,
    pages: [Option<Box<[u8; PAGE_SIZE]>>; TABLE_MAX_PAGES],
}

impl Table {
    pub fn new() -> Table {
        const EMPTY_PAGE: Option<Box<[u8; PAGE_SIZE]>> = None;
        Table {
            num_rows: 0,
            pages: [EMPTY_PAGE; TABLE_MAX_PAGES],
        }
    }
    pub fn row_slot_write(&mut self, row_num: usize) -> Result<&mut [u8], &str> {
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

    pub fn row_slot_read(&self, row_num: usize) -> Result<&[u8], &str> {
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
