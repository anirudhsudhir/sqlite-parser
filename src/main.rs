const USAGE: &str = "Usage: cargo r -- [path_to_db]";

fn main() {
    let mut args = std::env::args();

    args.next();
    let db_path = args.next().expect(USAGE);

    sqlite_parser::parse_db(&db_path);
}
