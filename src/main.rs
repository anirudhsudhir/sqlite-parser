mod types;

use std::{
    fs::File,
    io::{BufReader, Read},
};

const USAGE: &str = "Usage: cargo r -- [path_to_db]";

fn main() {
    let mut args = std::env::args();

    args.next();
    let db_path = args.next().expect(USAGE);

    parse_db(&db_path);
}

fn parse_db(db_path: &str) {
    let file = File::open(db_path).expect("failed to open db file");
    let mut buf_reader = BufReader::new(file);

    // The DB header is 100 bytes long
    let mut db_header_buf = [0u8; 100];
    let db_header = parse_db_header(&mut buf_reader, &mut db_header_buf);
    dbg!(&db_header);

    let mut page_buf = vec![0u8; db_header.page_size];
    loop {
        if let Err(e) = buf_reader.read_exact(&mut page_buf) {
            println!("{:?}", page_buf);

            //TODO: to fix
            println!("error: {e}");
            break;
        }

        println!("{:?}", page_buf);
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
            match ((db_header[16] as u16) << 8) + (db_header[17] as u16) {
                1 => 65536,
                size => size as usize,
            }
        },
        size_in_pages: ((db_header[28] as u32) << 24)
            + ((db_header[29] as u32) << 16)
            + ((db_header[30] as u32) << 8)
            + db_header[31] as u32,
        encoding: match db_header[59] {
            1 => types::Encoding::UTF8,
            2 => types::Encoding::UTF16LE,
            3 => types::Encoding::UTF16BE,
            _ => unreachable!("invalid encoding in db header"),
        },
        version_num: ((db_header[96] as u32) << 24)
            + ((db_header[97] as u32) << 16)
            + ((db_header[98] as u32) << 8)
            + db_header[99] as u32,
    }
}
