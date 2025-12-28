use crate::shape::Shape;
use crate::symbol::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wire {
    start: (i32, i32),
    end: (i32, i32),
}

impl Wire {
    fn from_asc_line(line: &str) -> Option<Wire> {
        // The format is WIRE {StartX} {StartY} {EndX} {EndY}
        let parts = line.splitn(5, ' ').collect::<Vec<&str>>();
        if parts.len() < 5 || parts[0] != "WIRE" {
            return None;
        }
        let start_x = parts[1].parse::<i32>().ok()?;
        let start_y = parts[2].parse::<i32>().ok()?;
        let end_x = parts[3].parse::<i32>().ok()?;
        let end_y = parts[4].parse::<i32>().ok()?;
        Some(Wire {
            start: (start_x, start_y),
            end: (end_x, end_y),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagType {
    Input,
    Output,
    Bidirectional,
}

impl FlagType {
    pub fn from_str(s: &str) -> Option<FlagType> {
        match s {
            "In" => Some(FlagType::Input),
            "Out" => Some(FlagType::Output),
            "BiDir" => Some(FlagType::Bidirectional),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flag {
    position: (i32, i32),
    name: String,
    io_type: Option<FlagType>,
}

impl Flag {
    fn from_asc_line(line: &str) -> Option<Flag> {
        // The format is FLAG {PositionX} {PositionY} {Name} [{IOType}]
        let parts = line.splitn(5, ' ').collect::<Vec<&str>>();
        if parts.len() < 4 || parts[0] != "FLAG" {
            return None;
        }
        let position_x = parts[1].parse::<i32>().ok()?;
        let position_y = parts[2].parse::<i32>().ok()?;
        let name = parts[3].to_string();
        Some(Flag {
            position: (position_x, position_y),
            name,
            io_type: None,
        })
    }

    fn add_io_type(&mut self, line: &str) {
        if self.io_type.is_some() {
            panic!("IO type already set for FLAG: {:?}", self);
        }
        let parts = line.splitn(5, ' ').collect::<Vec<&str>>();
        if parts.len() < 4 || parts[0] != "IOPIN" {
            return;
        }
        if let Some(io_type) = FlagType::from_str(parts[3]) {
            self.io_type = Some(io_type);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AscDocument {
    sheet_size: (i32, i32),
    wires: Vec<Wire>,
    flags: Vec<Flag>,
    symbols: Vec<Symbol>,
    shapes: Vec<Shape>,
}

impl AscDocument {
    fn check_version_line(line: &str) {
        if !line.starts_with("Version ") {
            panic!("Invalid ASC file: missing Version line");
        }
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        let version = parts[1];
        if version != "4" && version != "4.1" {
            panic!("Unsupported ASC version: {}", version);
        }
    }

    fn parse_sheet_line(line: &str) -> (i32, i32) {
        let parts = line.splitn(4, ' ').collect::<Vec<&str>>();
        if parts.len() != 4 {
            panic!("Invalid SHEET line: {}", line);
        }
        if parts[0] != "SHEET" {
            panic!("Invalid ASC file: missing SHEET line");
        }
        if parts[1] != "1" {
            panic!("Unsupported SHEET number: {}", parts[1]);
        }
        if let (Ok(width), Ok(height)) = (parts[2].parse::<i32>(), parts[3].parse::<i32>()) {
            (width, height)
        } else {
            panic!("Invalid SHEET line: {}", line);
        }
    }

    pub fn from_path(path: &str) -> std::io::Result<AscDocument> {
        let lines = crate::file_reader::read_file_lines(path)?;
        Ok(AscDocument::parse(lines))
    }

    pub fn parse(lines: Vec<String>) -> AscDocument {
        let mut wires = Vec::new();
        let mut flags = Vec::new();
        let mut symbols = Vec::new();
        let mut shapes = Vec::new();

        let mut file_lines = lines.into_iter();

        Self::check_version_line(file_lines.next().unwrap().as_str());

        let sheet_size = Self::parse_sheet_line(
            file_lines
                .next()
                .expect("Couldn't read second line")
                .as_str(),
        );

        for line in file_lines {
            let first_word = line.split_whitespace().next().unwrap_or("");
            match first_word {
                "Version" => {
                    println!("Ignoring redundant Version line: {}", line);
                }
                "DATAFLAG" => {
                    println!("Ignoring DATAFLAG line: {}", line);
                }
                "WIRE" => {
                    wires.push(
                        Wire::from_asc_line(&line)
                            .expect(&format!("Failed to parse WIRE line: {}", line)),
                    );
                }
                "FLAG" => {
                    flags.push(
                        Flag::from_asc_line(&line)
                            .expect(&format!("Failed to parse FLAG line: {}", line)),
                    );
                }
                "IOPIN" => {
                    if let Some(last_flag) = flags.last_mut() {
                        last_flag.add_io_type(&line);
                    } else {
                        panic!("IOPIN line without preceding FLAG: {}", line);
                    }
                }
                "SYMBOL" => {
                    if let Some(symbol) = Symbol::from_asc_line(&line) {
                        symbols.push(symbol);
                    } else {
                        panic!("Failed to parse SYMBOL line: {}", &line);
                    }
                }
                "WINDOW" | "SYMATTR" => {
                    if let Some(last_symbol) = symbols.last_mut() {
                        last_symbol.add_attribute(&line);
                    } else {
                        panic!("WINDOW line without preceding SYMBOL: {}", line);
                    }
                }
                _ => {
                    // Try to parse as shape
                    if let Some(shape) = Shape::parse_line(&line) {
                        shapes.push(shape);
                        continue;
                    }
                    if first_word.is_empty() {
                        continue; // skip empty lines
                    }
                    panic!("Unknown line type: {}", line);
                }
            }
        }

        AscDocument {
            sheet_size,
            wires,
            flags,
            symbols,
            shapes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_wire() {
        let line = "WIRE 480 720 640 720";
        let wire = Wire::from_asc_line(line).expect("Failed to parse WIRE line");
        assert_eq!(wire.start, (480, 720));
        assert_eq!(wire.end, (640, 720));
    }
    #[test]
    fn test_parse_flag() {
        // cspell: disable-next-line
        let line = "FLAG 640 720 Vout";
        let flag = Flag::from_asc_line(line).expect("Failed to parse FLAG line");
        assert_eq!(flag.position, (640, 720));
        // cspell: disable-next-line
        assert_eq!(flag.name, "Vout");
        assert_eq!(flag.io_type, None);
    }

    #[test]
    fn test_asc_example() {
        let document = AscDocument::from_path("test_files/complex_sample.asc")
            .expect("Failed to read ASC file");
        assert_eq!(document.wires.len(), 1);
        assert_eq!(document.flags.len(), 6);
        assert_eq!(document.symbols.len(), 11);
    }

    #[test]
    fn test_all_asc_files_from_examples() {
        let user_name = std::env::var("USER").unwrap();
        // cspell: disable-next-line
        let lib_path = format!("/home/{user}/.local/share/ltspice/dosdevices/c:/users/{user}/AppData/Local/LTspice/examples", user=user_name);
        // Recurse through all .asy files in the lib_path also in subdirectories
        let asc_files = glob::glob(&format!("{}/**/*.asc", lib_path))
            .unwrap()
            .collect::<Vec<_>>();
        let total_files = asc_files.len();
        println!("Found {} ASC files in LTspice lib", total_files);
        let start_time = std::time::Instant::now();
        for (current_index, entry) in asc_files.iter().enumerate() {
            let path = entry.as_ref().unwrap();
            print!(
                "\r{current_index:5} of {total_files}: Testing ASC file: {}    ",
                path.to_str().unwrap(),
                current_index = current_index + 1
            );
            let _asc_file =
                AscDocument::from_path(path.to_str().unwrap()).expect("Failed to read ASC file");
        }
        let duration = start_time.elapsed();
        println!();
        println!("All ASC files parsed successfully in {:?}", duration);
    }
}
