use std::fs;
use std::io;

pub fn write_to_file(filename: &str, content: &str) -> io::Result<()> {
    fs::write(filename, content)
}