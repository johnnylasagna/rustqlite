use crate::storage::{PAGE_SIZE, ROW_SIZE, Row, Table};

// Common Node Header Layout
const NODE_TYPE_SIZE: usize = 1;
const NODE_TYPE_OFFSET: usize = 0;
const IS_ROOT_SIZE: usize = 1;
const IS_ROOT_OFFSET: usize = NODE_TYPE_SIZE;
const PARENT_POINTER_SIZE: usize = 4;
const PARENT_POINTER_OFFSET: usize = IS_ROOT_OFFSET + IS_ROOT_SIZE;
const COMMON_NODE_HEADER_SIZE: usize = NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_POINTER_SIZE;

// Leaf Node Header Layout
const LEAF_NODE_NUM_CELLS_SIZE: usize = 4;
const LEAF_NODE_NUM_CELLS_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
const LEAF_NODE_HEADER_SIZE: usize = COMMON_NODE_HEADER_SIZE + LEAF_NODE_NUM_CELLS_SIZE;

// Leaf Node Body Layout
const LEAF_NODE_KEY_SIZE: usize = 4;
const LEAF_NODE_KEY_OFFSET: usize = 0;
const LEAF_NODE_VALUE_SIZE: usize = ROW_SIZE;
const LEAF_NODE_VALUE_OFFSET: usize = LEAF_NODE_KEY_OFFSET + LEAF_NODE_KEY_SIZE;
const LEAF_NODE_CELL_SIZE: usize = LEAF_NODE_KEY_SIZE + LEAF_NODE_VALUE_SIZE;
const LEAF_NODE_SPACE_FOR_CELLS: usize = PAGE_SIZE - LEAF_NODE_HEADER_SIZE;
const LEAF_NODE_MAX_CELLS: usize = LEAF_NODE_SPACE_FOR_CELLS / LEAF_NODE_CELL_SIZE;

// Node access functions
pub fn leaf_node_num_cells(node: &[u8]) -> u32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(
        &node[LEAF_NODE_NUM_CELLS_OFFSET..LEAF_NODE_NUM_CELLS_OFFSET + LEAF_NODE_NUM_CELLS_SIZE],
    );
    u32::from_le_bytes(bytes)
}

pub fn set_leaf_node_num_cells(node: &mut [u8], num_cells: u32) {
    node[LEAF_NODE_NUM_CELLS_OFFSET..LEAF_NODE_NUM_CELLS_OFFSET + LEAF_NODE_NUM_CELLS_SIZE]
        .copy_from_slice(&num_cells.to_le_bytes());
}

pub fn initialize_leaf_node(node: &mut [u8]) {
    set_leaf_node_num_cells(node, 0);
}

fn cell_offset(cell_num: usize) -> usize {
    LEAF_NODE_HEADER_SIZE + cell_num * LEAF_NODE_CELL_SIZE
}

/// Cursor
pub struct Cursor<'a> {
    table: &'a mut Table,
    pub page_num: usize,
    pub cell_num: usize,
    pub end_of_table: bool,
}

impl<'a> Cursor<'a> {
    pub fn start(table: &'a mut Table) -> Result<Cursor<'a>, &'static str> {
        let page_num = table.root_page_num;
        let node = table.pager.get_page(page_num)?;
        let num_cells = leaf_node_num_cells(node);
        let end_of_table = num_cells == 0;
        Ok(Cursor {
            table,
            page_num: 0,
            cell_num: 0,
            end_of_table,
        })
    }

    pub fn end(table: &'a mut Table) -> Result<Cursor<'a>, &'static str> {
        let page_num = table.root_page_num;
        let node = table.pager.get_page(page_num)?;
        let num_cells = leaf_node_num_cells(node);

        Ok(Cursor {
            table,
            page_num,
            cell_num: num_cells as usize,
            end_of_table: true,
        })
    }

    pub fn value(&mut self) -> Result<&mut [u8], &'static str> {
        let page_num = self.page_num;
        let cell_num = self.cell_num;
        let offset = cell_offset(cell_num);

        let page = self.table.pager.get_page(page_num)?;
        Ok(&mut page[offset + LEAF_NODE_KEY_SIZE..offset + LEAF_NODE_CELL_SIZE])
    }

    pub fn advance(&mut self) -> Result<(), &'static str> {
        let page_num = self.page_num;
        let node = self.table.pager.get_page(page_num)?;

        self.cell_num += 1;
        if self.cell_num >= leaf_node_num_cells(node) as usize {
            self.end_of_table = true;
        }

        Ok(())
    }
}

// B-tree functions

pub fn leaf_node_insert(cursor: &mut Cursor, key: u32, value: &Row) -> Result<(), &'static str> {
    let page_num = cursor.page_num;
    let node = cursor.table.pager.get_page(page_num)?;
    let num_cells = leaf_node_num_cells(node) as usize;

    if num_cells >= LEAF_NODE_MAX_CELLS {
        return Err("Need to implement splitting a leaf node.");
    }

    if cursor.cell_num < num_cells {
        let start = cell_offset(cursor.cell_num);
        let end = cell_offset(num_cells);
        let dest = cell_offset(cursor.cell_num + 1);
        node.copy_within(start..end, dest);
    }

    set_leaf_node_num_cells(node, (num_cells + 1) as u32);

    let cell_off = cell_offset(cursor.cell_num);
    node[cell_off..cell_off + LEAF_NODE_KEY_SIZE].copy_from_slice(&key.to_le_bytes());
    value.serialize(&mut node[cell_off + LEAF_NODE_KEY_SIZE..cell_off + LEAF_NODE_CELL_SIZE]);

    Ok(())
}
