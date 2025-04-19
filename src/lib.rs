mod parser;

use std::{
    fs::File,
    io::{BufReader, Read},
};

use parser::types;

/// Parses bytes in the Big-Endian ordering
fn be_parser(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .enumerate()
        .map(|(i, byte)| (*byte as usize) << (8 * (bytes.len() - i - 1)))
        .sum()
}

/// Parses varints and returns it along with its size in bytes
fn varint_parser(bytes: &[u8], cursor: usize) -> (u64, usize) {
    let mut val = 0;
    let mut varint_len = 0;

    // First 8 variable bytes, with 7 bits for value and MSB for varint counts
    for i in 0..=7 {
        let byte = bytes[cursor + i];
        val = (val << 7) + ((byte as u64) & 0b01111111);
        varint_len += 1;

        if byte & 0b10000000 != 0b10000000 {
            return (val, varint_len);
        }
    }

    // 9th byte
    ((val << 8) + bytes[cursor + 8] as u64, varint_len + 1)
}

pub fn parse_db(db_path: &str) {
    let file = File::open(db_path).expect("failed to open db file");
    let mut buf_reader = BufReader::new(file);

    // The DB header is 100 bytes long
    let mut db_header_buf = [0u8; 100];
    let db_header = parse_db_header(&mut buf_reader, &mut db_header_buf);
    dbg!(&db_header);

    let mut pages_remaining = db_header.size_in_pages;
    // root page = page_size - 100 (DB header)
    let mut root_page_buf = vec![0u8; db_header.page_size - 100];

    if let Err(e) = buf_reader.read_exact(&mut root_page_buf) {
        println!("error while reading root page: {e}");
        return;
    }

    pages_remaining -= 1;

    let mut page_buf = vec![0u8; db_header.page_size];
    while pages_remaining > 0 {
        println!("\n\nREMAINING PAGES = {pages_remaining} , READING NEXT PAGE:");
        if let Err(e) = buf_reader.read_exact(&mut page_buf) {
            println!("error while reading pages: {e}");
            break;
        }

        parse_btree_page(&mut page_buf);
        pages_remaining -= 1;
    }
}

fn parse_btree_page(page: &mut [u8]) {
    println!();

    let page_type = match page[0] {
        2 => {
            println!("encountered interior index b-tree page");
            types::BTreePage::InteriorIndex(parse_interior_page_header(page))
        }
        5 => {
            println!("encountered interior table b-tree page");
            types::BTreePage::InteriorTable(parse_interior_page_header(page))
        }
        10 => {
            println!("encountered leaf index b-tree page");
            types::BTreePage::LeafIndex(parse_leaf_page_header(page))
        }
        13 => {
            println!("encountered leaf table b-tree page");
            types::BTreePage::LeafTable(parse_leaf_page_header(page))
        }
        _ => unreachable!("invalid b-tree page type"),
    };
    println!();

    //TODO: remore linter suppression
    #[allow(clippy::single_match)]
    match page_type {
        types::BTreePage::LeafTable(header) => {
            dbg!(&header);
            parse_leaf_table_page(page, header);
        }
        _ => {
            println!("non-leaf table");
        }
    }
}

fn parse_leaf_table_page(page: &[u8], header: types::LeafPageHeader) {
    println!(
        "total leaf cells = {}, starting cell parsing:",
        header.interior_page_header.num_cells
    );

    let mut page_cursor = header.interior_page_header.cell_content_start as usize;
    for cell_count in 1..=header.interior_page_header.num_cells {
        println!(
            "\nparsing cell = {cell_count}, remaining cell count = {}",
            header.interior_page_header.num_cells - cell_count
        );

        // Parsing number of bytes of payload - varint
        let (payload_size, varint_len) = varint_parser(page, page_cursor);
        page_cursor += varint_len;
        // Integer key - rowid
        let (rowid, varint_len) = varint_parser(page, page_cursor);
        page_cursor += varint_len;

        println!("payload size = {payload_size}");
        println!("rowid = {rowid}");

        let record = &page[page_cursor..page_cursor + payload_size as usize];
        page_cursor += payload_size as usize;
        println!("record = {:?}", record);
        parse_record(record);
    }
}

fn parse_record(record: &[u8]) {
    let mut record_cursor = 0;

    let (record_header_size, varint_len) = varint_parser(record, record_cursor);
    record_cursor += varint_len;

    dbg!(record_header_size, record_cursor, varint_len);
    if varint_len > record_header_size as usize {
        return;
    }
    let mut remaining_header_len = record_header_size as usize - varint_len;
    let mut serial_types = Vec::new();

    while remaining_header_len > 0 {
        let (serial_type, varint_len) = varint_parser(record, record_cursor);

        record_cursor += varint_len;
        serial_types.push(serial_type);

        remaining_header_len -= varint_len;
    }

    dbg!(&serial_types);

    for serial_type in serial_types {
        print!("column value in record: ");
        match serial_type {
            0 => println!("NULL value"),

            1 => {
                println!(
                    "8-bit integer = {}",
                    be_parser(&record[record_cursor..=record_cursor])
                );
                record_cursor += 1;
            }

            2 => {
                println!(
                    "16-bit integer = {}",
                    be_parser(&record[record_cursor..=record_cursor + 1])
                );
                record_cursor += 2;
            }

            3 => {
                println!(
                    "24-bit integer = {}",
                    be_parser(&record[record_cursor..=record_cursor + 2])
                );
                record_cursor += 3;
            }

            4 => {
                println!(
                    "32-bit integer = {}",
                    be_parser(&record[record_cursor..=record_cursor + 3])
                );
                record_cursor += 4;
            }

            5 => {
                println!(
                    "48-bit integer = {}",
                    be_parser(&record[record_cursor..=record_cursor + 5])
                );
                record_cursor += 6;
            }

            6 => {
                println!(
                    "64-bit integer = {}",
                    be_parser(&record[record_cursor..=record_cursor + 7])
                );
                record_cursor += 8;
            }

            7 => {
                println!(
                    "64-bit floating point = {}",
                    be_parser(&record[record_cursor..=record_cursor + 7])
                );
                record_cursor += 8;
            }

            8 => {
                println!("integer value 0",);
            }

            9 => {
                println!("integer value 1",);
            }

            st if st >= 12 && st % 2 == 0 => {
                let blob_len = (st as usize - 12) / 2;
                println!(
                    "blob type = {:?}",
                    &record[record_cursor..record_cursor + blob_len]
                );
                record_cursor += blob_len;
            }
            st if st >= 13 && st % 2 == 1 => {
                let str_len = (st as usize - 13) / 2;
                println!(
                    "string type = {}",
                    String::from_utf8(record[record_cursor..record_cursor + str_len].to_vec())
                        .expect("failed to parse string from utf8 record in cell"),
                );
                record_cursor += str_len;
            }
            _ => unreachable!("invalid record serial type!"),
        }

        println!("record_cursor = {record_cursor}");
    }
}

fn parse_db_header(buf_reader: &mut BufReader<File>, db_header: &mut [u8]) -> types::DBHeader {
    buf_reader
        .read_exact(db_header)
        .expect("failed to read db header: 100 bytes");

    types::DBHeader {
        header: String::from_utf8(db_header[0..15].to_vec())
            .expect("failed to parse header string from db header"),
        page_size: {
            match be_parser(&db_header[16..=17]) as u16 {
                1 => 65536,
                size => size as usize,
            }
        },
        size_in_pages: be_parser(&db_header[28..=31]) as u32,
        encoding: match db_header[59] {
            1 => types::Encoding::UTF8,
            2 => types::Encoding::UTF16LE,
            3 => types::Encoding::UTF16BE,
            _ => unreachable!("invalid encoding in db header"),
        },
        version_num: be_parser(&db_header[96..=99]) as u32,
    }
}

fn parse_interior_page_header(page: &[u8]) -> types::InteriorPageHeader {
    types::InteriorPageHeader {
        freeblock_start: be_parser(&page[1..=2]) as u16,
        num_cells: be_parser(&page[3..=4]) as u16,
        cell_content_start: be_parser(&page[5..=6]) as u16,
        fragmented_bytes: be_parser(&page[7..=7]) as u8,
    }
}

fn parse_leaf_page_header(page: &[u8]) -> types::LeafPageHeader {
    types::LeafPageHeader {
        interior_page_header: parse_interior_page_header(page),
        rightmost_pointer: be_parser(&page[8..=12]) as u32,
    }
}
