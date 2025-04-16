#![allow(dead_code)]

#[derive(Debug)]
pub enum Encoding {
    UTF8,
    UTF16LE,
    UTF16BE,
}

#[derive(Debug)]
pub struct DBHeader {
    pub header: String,
    pub page_size: usize,
    pub size_in_pages: u32,
    pub encoding: Encoding,
    pub version_num: u32,
}
