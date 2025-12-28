//! Read a file and return the lines as a vector of strings
//! Removes any null bytes and carriage returns from the lines

use std::path::PathBuf;

fn read_utf16_file(path: &PathBuf) -> std::io::Result<Vec<String>> {
    let content = std::fs::read(path);
    if content.is_err() {
        return Err(content.err().unwrap());
    }
    let content = content.unwrap();
    let (decoded, _, had_errors) = encoding_rs::UTF_16LE.decode(&content);
    if had_errors {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to decode UTF-16BE file",
        ));
    }
    Ok(decoded
        .lines()
        .map(|line| line.to_string().replace("\r", ""))
        .collect())
}

fn read_utf8_file(path: &PathBuf) -> std::io::Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    let lines = content.lines();
    Ok(lines
        .map(|line| line.to_string().replace("\r", ""))
        .collect())
}

fn read_windows1252_file(path: &PathBuf) -> std::io::Result<Vec<String>> {
    let content = std::fs::read(path);
    if content.is_err() {
        return Err(content.err().unwrap());
    }
    let content = content.unwrap();
    let (decoded, _, had_errors) = encoding_rs::WINDOWS_1252.decode(&content);
    if had_errors {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to decode Latin1 file",
        ));
    }
    Ok(decoded
        .lines()
        .map(|line| line.to_string().replace("\r", ""))
        .collect())
}

fn check_null_characters(lines: &Vec<String>) -> bool {
    for line in lines {
        if line.contains('\0') {
            return true;
        }
    }
    false
}

pub fn read_file_lines(path: &PathBuf) -> std::io::Result<Vec<String>> {
    // Check if a file exists at the given path
    if !std::path::Path::new(path).exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", path.display()),
        ));
    }

    let windows1252 = read_windows1252_file(path);
    // Check if the windows1252 read was successful and check if it contains any invalid characters
    if windows1252.is_ok() && !check_null_characters(&windows1252.as_ref().unwrap()) {
        return windows1252;
    }

    // println!("Failed to read as Windows-1252, trying UTF-8...");

    let utf_8 = read_utf8_file(path);
    // Check if the utf-8 read was successful and check if it contains any invalid characters
    if utf_8.is_ok() && !check_null_characters(&utf_8.as_ref().unwrap()) {
        return utf_8;
    }

    // println!("Failed to read as UTF-8, trying UTF-16LE...");

    let utf_16 = read_utf16_file(path);
    // Check if the utf-16 read was successful and check if it contains any invalid characters
    if utf_16.is_ok() && !check_null_characters(&utf_16.as_ref().unwrap()) {
        return utf_16;
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Failed to read file in UTF-8, UTF-16LE, or Windows-1252 encoding",
    ))
}
