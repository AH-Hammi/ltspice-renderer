//! Read a file and return the lines as a vector of strings
//! Removes any null bytes and carriage returns from the lines

fn read_utf16_file(path: &str) -> std::io::Result<Vec<String>> {
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
    let lines = decoded
        .lines()
        .map(|line| line.replace("\0", "").replace("\r", ""))
        .collect();
    Ok(lines)
}

fn read_utf8_file(path: &str) -> std::io::Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    let lines = content
        .lines()
        .map(|line| line.replace("\0", "").replace("\r", ""))
        .collect();
    Ok(lines)
}

fn read_latin1_file(path: &str) -> std::io::Result<Vec<String>> {
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
    let lines = decoded
        .lines()
        .map(|line| line.replace("\0", "").replace("\r", ""))
        .collect();
    Ok(lines)
}

pub fn read_file_lines(path: &str) -> std::io::Result<Vec<String>> {
    if let Ok(lines) = read_utf8_file(path) {
        return Ok(lines);
    }

    if let Ok(lines) = read_latin1_file(path) {
        return Ok(lines);
    }

    if let Ok(lines) = read_utf16_file(path) {
        return Ok(lines);
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Failed to read file in UTF-8, UTF-16LE, or Latin1 encoding",
    ))
}
