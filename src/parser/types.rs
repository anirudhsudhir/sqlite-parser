#![allow(dead_code)]

#[derive(Debug)]
pub enum Encoding {
    UTF8,
    UTF16LE,
    UTF16BE,
}

#[derive(Debug)]
pub enum BTreePage {
    InteriorTable(InteriorPageHeader),
    InteriorIndex(InteriorPageHeader),
    LeafTable(LeafPageHeader),
    LeafIndex(LeafPageHeader),
}

#[derive(Debug)]
pub struct DBHeader {
    pub header: String,
    pub page_size: usize,
    pub size_in_pages: u32,
    pub encoding: Encoding,
    pub version_num: u32,
}

#[derive(Debug)]
pub struct InteriorPageHeader {
    pub freeblock_start: u16,
    pub num_cells: u16,
    pub cell_content_start: u16,
    pub fragmented_bytes: u8,
}

#[derive(Debug)]
pub struct LeafPageHeader {
    pub interior_page_header: InteriorPageHeader,
    pub rightmost_pointer: u32,
}
