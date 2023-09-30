use std::io;
use std::io::Write;

pub fn get_input(mes: String) -> String {
    print!("{mes}");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
    buf.trim().to_string()
}
