use crate::storage::{PAGE_SIZE, Pager, ROW_SIZE, Row, Table};
use std::io::{self, Write};

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

pub const LEAF_NODE_RIGHT_SPLIT_COUNT: usize = (LEAF_NODE_MAX_CELLS + 1) / 2;
pub const LEAF_NODE_LEFT_SPLIT_COUNT: usize = (LEAF_NODE_MAX_CELLS + 1) - LEAF_NODE_RIGHT_SPLIT_COUNT;

// Internal Node Header Layout
const INTERNAL_NODE_NUM_KEYS_SIZE: usize = 4;
const INTERNAL_NODE_NUM_KEYS_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
const INTERNAL_NODE_RIGHT_CHILD_SIZE: usize = 4;
const INTERNAL_NODE_RIGHT_CHILD_OFFSET: usize = INTERNAL_NODE_NUM_KEYS_OFFSET + INTERNAL_NODE_NUM_KEYS_SIZE;
const INTERNAL_NODE_HEADER_SIZE: usize =
    COMMON_NODE_HEADER_SIZE + INTERNAL_NODE_NUM_KEYS_SIZE + INTERNAL_NODE_RIGHT_CHILD_SIZE;

// Internal Node Body Layout
const INTERNAL_NODE_KEY_SIZE: usize = 4;
const INTERNAL_NODE_CHILD_SIZE: usize = 4;
const INTERNAL_NODE_CELL_SIZE: usize = INTERNAL_NODE_CHILD_SIZE + INTERNAL_NODE_KEY_SIZE;

// Command outputs
pub fn print_constants() {
    println!("ROW_SIZE: {}", ROW_SIZE);
    println!("COMMON_NODE_HEADER_SIZE: {}", COMMON_NODE_HEADER_SIZE);
    println!("LEAF_NODE_HEADER_SIZE: {}", LEAF_NODE_HEADER_SIZE);
    println!("LEAF_NODE_CELL_SIZE: {}", LEAF_NODE_CELL_SIZE);
    println!("LEAF_NODE_SPACE_FOR_CELLS: {}", LEAF_NODE_SPACE_FOR_CELLS);
    println!("LEAF_NODE_MAX_CELLS: {}", LEAF_NODE_MAX_CELLS);
}

pub fn indent(level: usize) {
    for _ in 0..level {
        print!(" ");
    }
    io::stdout().flush().unwrap();
}

pub fn print_tree(pager: &mut Pager, page_num: usize, indentation_level: usize) -> Result<(), &'static str> {
    let node_type = {
        let node = pager.get_page(page_num)?;
        get_node_type(node)
    };

    match node_type {
        NodeType::Leaf => {
            let node = pager.get_page(page_num)?;
            let num_keys = leaf_node_num_cells(node);
            indent(indentation_level);
            println!("- leaf (size {})", num_keys);
            for i in 0..num_keys {
                indent(indentation_level + 1);
                println!("- {}", get_leaf_node_key(node, i as usize));
            }
        }
        NodeType::Internal => {
            let (num_keys, children_and_keys, right_child) = {
                let node = pager.get_page(page_num)?;
                let num_keys = internal_node_num_keys(node);

                let mut extracted = Vec::new();
                for i in 0..num_keys {
                    extracted.push((internal_node_child(node, i as usize)?, internal_node_key(node, i as usize)));
                }

                let right_child = internal_node_right_child(node);
                (num_keys, extracted, right_child)
            };

            indent(indentation_level);
            println!("- internal (size {})", num_keys);

            for (child, key) in children_and_keys {
                print_tree(pager, child as usize, indentation_level + 1)?;
                indent(indentation_level + 1);
                println!("- key {}", key);
            }

            print_tree(pager, right_child as usize, indentation_level + 1)?;
        }
    }

    Ok(())
}

pub fn print_leaf_node(node: &[u8]) {
    let num_cells = leaf_node_num_cells(node);
    println!("leaf size {}", num_cells);

    for i in 0..num_cells {
        let offset = cell_offset(i as usize);
        let mut key_bytes = [0u8; 4];

        key_bytes.copy_from_slice(&node[offset..offset + LEAF_NODE_KEY_SIZE]);
        let key = u32::from_le_bytes(key_bytes);

        println!("  - {} : {}", i, key);
    }
}

// Node access functions
#[derive(PartialEq)]
pub enum NodeType {
    Internal,
    Leaf,
}

pub fn get_node_type(node: &[u8]) -> NodeType {
    match node[NODE_TYPE_OFFSET] {
        0 => NodeType::Internal,
        1 => NodeType::Leaf,
        _ => panic!("Unknown node type"),
    }
}

pub fn set_node_type(node: &mut [u8], node_type: NodeType) {
    node[NODE_TYPE_OFFSET] = match node_type {
        NodeType::Internal => 0,
        NodeType::Leaf => 1,
    };
}

pub fn is_node_root(node: &[u8]) -> bool {
    node[IS_ROOT_OFFSET] == 1
}

pub fn set_node_root(node: &mut [u8], is_root: bool) {
    node[IS_ROOT_OFFSET] = is_root as u8;
}

// Leaf node functions
pub fn initialize_leaf_node(node: &mut [u8]) {
    set_node_type(node, NodeType::Leaf);
    set_node_root(node, false);
    set_leaf_node_num_cells(node, 0);
}

pub fn leaf_node_num_cells(node: &[u8]) -> u32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&node[LEAF_NODE_NUM_CELLS_OFFSET..LEAF_NODE_NUM_CELLS_OFFSET + LEAF_NODE_NUM_CELLS_SIZE]);
    u32::from_le_bytes(bytes)
}

pub fn set_leaf_node_num_cells(node: &mut [u8], num_cells: u32) {
    node[LEAF_NODE_NUM_CELLS_OFFSET..LEAF_NODE_NUM_CELLS_OFFSET + LEAF_NODE_NUM_CELLS_SIZE]
        .copy_from_slice(&num_cells.to_le_bytes());
}

pub fn get_leaf_node_key(node: &[u8], cell_num: usize) -> u32 {
    let offset = cell_offset(cell_num);
    let mut key_bytes = [0u8; 4];
    key_bytes.copy_from_slice(&node[offset..offset + LEAF_NODE_KEY_SIZE]);
    u32::from_le_bytes(key_bytes)
}

pub fn leaf_node_value(node: &mut [u8], cell_num: usize) -> &mut [u8] {
    let offset = cell_offset(cell_num);
    &mut node[offset + LEAF_NODE_KEY_SIZE..offset + LEAF_NODE_CELL_SIZE]
}

fn cell_offset(cell_num: usize) -> usize {
    LEAF_NODE_HEADER_SIZE + cell_num * LEAF_NODE_CELL_SIZE
}

// Internal node functions
fn internal_node_num_keys(node: &[u8]) -> u32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(
        &node[INTERNAL_NODE_NUM_KEYS_OFFSET..INTERNAL_NODE_NUM_KEYS_OFFSET + INTERNAL_NODE_NUM_KEYS_SIZE],
    );
    u32::from_le_bytes(bytes)
}

pub fn set_internal_node_num_keys(node: &mut [u8], num_keys: u32) {
    node[INTERNAL_NODE_NUM_KEYS_OFFSET..INTERNAL_NODE_NUM_KEYS_OFFSET + INTERNAL_NODE_NUM_KEYS_SIZE]
        .copy_from_slice(&num_keys.to_le_bytes());
}

pub fn internal_node_right_child(node: &[u8]) -> u32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(
        &node[INTERNAL_NODE_RIGHT_CHILD_OFFSET..INTERNAL_NODE_RIGHT_CHILD_OFFSET + INTERNAL_NODE_RIGHT_CHILD_SIZE],
    );
    u32::from_le_bytes(bytes)
}

pub fn set_internal_node_right_child(node: &mut [u8], child_num: u32) {
    node[INTERNAL_NODE_RIGHT_CHILD_OFFSET..INTERNAL_NODE_RIGHT_CHILD_OFFSET + INTERNAL_NODE_RIGHT_CHILD_SIZE]
        .copy_from_slice(&child_num.to_le_bytes());
}

fn internal_node_cell_offset(cell_num: usize) -> usize {
    INTERNAL_NODE_HEADER_SIZE + cell_num * INTERNAL_NODE_CELL_SIZE
}

pub fn internal_node_child(node: &[u8], child_num: usize) -> Result<u32, &'static str> {
    let num_keys = internal_node_num_keys(node) as usize;
    if child_num > num_keys {
        Err("Tried to access child_num > num_keys")
    } else if child_num == num_keys {
        Ok(internal_node_right_child(node))
    } else {
        let offset = internal_node_cell_offset(child_num);
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&node[offset..offset + INTERNAL_NODE_CHILD_SIZE]);
        Ok(u32::from_le_bytes(bytes))
    }
}

pub fn set_internal_node_child(node: &mut [u8], child_num: usize, child_page_num: u32) -> Result<(), &'static str> {
    let num_keys = internal_node_num_keys(node) as usize;
    if child_num > num_keys {
        return Err("Tried to access child_num > num_keys");
    } else if child_num == num_keys {
        set_internal_node_right_child(node, child_page_num);
    } else {
        let offset = internal_node_cell_offset(child_num);
        node[offset..offset + INTERNAL_NODE_CHILD_SIZE].copy_from_slice(&child_page_num.to_le_bytes());
    }
    Ok(())
}

pub fn internal_node_key(node: &[u8], key_num: usize) -> u32 {
    let offset = internal_node_cell_offset(key_num) + INTERNAL_NODE_CHILD_SIZE;
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&node[offset..offset + INTERNAL_NODE_KEY_SIZE]);
    u32::from_le_bytes(bytes)
}

pub fn set_internal_node_key(node: &mut [u8], key_num: usize, key: u32) {
    let offset = internal_node_cell_offset(key_num) + INTERNAL_NODE_CHILD_SIZE;
    node[offset..offset + INTERNAL_NODE_KEY_SIZE].copy_from_slice(&key.to_le_bytes());
}

pub fn get_node_max_key(node: &[u8]) -> u32 {
    match get_node_type(node) {
        NodeType::Internal => {
            let num_keys = internal_node_num_keys(node) as usize;
            internal_node_key(node, num_keys - 1)
        }
        NodeType::Leaf => {
            let num_cells = leaf_node_num_cells(node) as usize;
            get_leaf_node_key(node, num_cells - 1)
        }
    }
}

pub fn initialize_internal_node(node: &mut [u8]) {
    set_node_type(node, NodeType::Internal);
    set_node_root(node, false);
    set_internal_node_num_keys(node, 0);
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
        Ok(Cursor { table, page_num: 0, cell_num: 0, end_of_table })
    }

    pub fn find(table: &'a mut Table, key: u32) -> Result<Cursor<'a>, &'static str> {
        let root_page_num = table.root_page_num;
        let root_node = table.pager.get_page(root_page_num)?;

        if get_node_type(root_node) == NodeType::Leaf {
            Self::leaf_node_find(table, root_page_num, key)
        } else {
            Err("Need to implement searching an internal node")
        }
    }

    pub fn leaf_node_find(table: &'a mut Table, page_num: usize, key: u32) -> Result<Cursor<'a>, &'static str> {
        let node = table.pager.get_page(page_num)?;
        let num_cells = leaf_node_num_cells(node) as usize;

        let mut min_index = 0;
        let mut one_past_max_index = num_cells;

        while one_past_max_index != min_index {
            let index = (min_index + one_past_max_index) / 2;
            let key_at_index = get_leaf_node_key(node, index);

            if key == key_at_index {
                min_index = index;
                break;
            }

            if key < key_at_index {
                one_past_max_index = index;
            } else {
                min_index = index + 1;
            }
        }

        Ok(Cursor { table, page_num, cell_num: min_index, end_of_table: false })
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
        leaf_node_split_and_insert(cursor, key, value)?;
        return Ok(());
    }

    if cursor.cell_num < num_cells {
        let key_at_index = get_leaf_node_key(node, cursor.cell_num);
        if key_at_index == key {
            return Err("Duplicate Key");
        }

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

pub fn leaf_node_split_and_insert(cursor: &mut Cursor, key: u32, value: &Row) -> Result<(), &'static str> {
    let old_page_num = cursor.page_num;
    let mut old_node_temp = [0u8; PAGE_SIZE];
    {
        let old_node = cursor.table.pager.get_page(old_page_num)?;
        old_node_temp.copy_from_slice(old_node);
    }

    let new_page_num = cursor.table.pager.get_unused_page_number();
    {
        let new_node = cursor.table.pager.get_page(new_page_num)?;
        initialize_leaf_node(new_node);
    }

    for i in (0..LEAF_NODE_MAX_CELLS).rev() {
        let dest_page_num = if i >= LEAF_NODE_LEFT_SPLIT_COUNT {
            new_page_num
        } else {
            old_page_num
        };

        let index_within_node = i % LEAF_NODE_LEFT_SPLIT_COUNT;

        let mut cell_bytes = [0u8; LEAF_NODE_CELL_SIZE];

        if i == cursor.cell_num {
            cell_bytes[0..LEAF_NODE_KEY_SIZE].copy_from_slice(&key.to_le_bytes());
            value.serialize(&mut cell_bytes[LEAF_NODE_KEY_SIZE..LEAF_NODE_CELL_SIZE]);
        } else {
            let src_index = if i > cursor.cell_num { i - 1 } else { i };
            let offset = cell_offset(src_index);
            cell_bytes.copy_from_slice(&old_node_temp[offset..offset + LEAF_NODE_CELL_SIZE]);
        }

        let dest_node = cursor.table.pager.get_page(dest_page_num)?;
        let dest_offset = cell_offset(index_within_node);
        dest_node[dest_offset..dest_offset + LEAF_NODE_CELL_SIZE].copy_from_slice(&cell_bytes);
    }

    let old_node = cursor.table.pager.get_page(old_page_num)?;
    set_leaf_node_num_cells(old_node, LEAF_NODE_LEFT_SPLIT_COUNT as u32);
    let is_root = is_node_root(old_node);

    let new_node = cursor.table.pager.get_page(new_page_num)?;
    set_leaf_node_num_cells(new_node, LEAF_NODE_RIGHT_SPLIT_COUNT as u32);

    if is_root {
        create_new_root(cursor.table, new_page_num)?;
        Ok(())
    } else {
        Err("Need to implement updating parent after split")
    }
}

pub fn create_new_root(table: &mut Table, right_child_page_num: usize) -> Result<(), &'static str> {
    let root_page_num = table.root_page_num;

    let mut old_root_temp = [0u8; PAGE_SIZE];
    {
        let root_node = table.pager.get_page(root_page_num)?;
        old_root_temp.copy_from_slice(root_node);
    }

    let left_child_page_num = table.pager.get_unused_page_number();
    {
        let left_child_node = table.pager.get_page(left_child_page_num)?;
        left_child_node.copy_from_slice(&old_root_temp);
        set_node_root(left_child_node, false);
    }

    let left_child_max_key = get_node_max_key(&old_root_temp);

    {
        let root_node = table.pager.get_page(root_page_num)?;
        initialize_internal_node(root_node);
        set_node_root(root_node, true);
        set_internal_node_num_keys(root_node, 1);

        set_internal_node_child(root_node, 0, left_child_page_num as u32)?;
        set_internal_node_key(root_node, 0, left_child_max_key);

        set_internal_node_right_child(root_node, right_child_page_num as u32);
    }

    Ok(())
}
