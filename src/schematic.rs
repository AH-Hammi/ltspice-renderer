#![allow(dead_code)]

use std::path::PathBuf;

use crate::shape::Shape;
use crate::symbol::{self, Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wire {
    pub start: (i32, i32),
    pub end: (i32, i32),
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
    pub position: (i32, i32),
    pub name: String,
    pub io_type: Option<FlagType>,
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
pub struct Schematic {
    pub sheet_size: (i32, i32),
    pub wires: Vec<Wire>,
    pub flags: Vec<Flag>,
    pub symbols: Vec<Symbol>,
    pub shapes: Vec<Shape>,
    pub symbol_loader: symbol::SymbolLoader,
}

impl Schematic {
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

    pub fn from_path_with_symbol_loader(
        path: &PathBuf,
        symbol_loader: symbol::SymbolLoader,
    ) -> std::io::Result<Schematic> {
        let lines = crate::file_reader::read_file_lines(path)?;
        Ok(Schematic::parse(lines, symbol_loader))
    }

    pub fn from_path(path: &PathBuf) -> std::io::Result<Schematic> {
        let lines = crate::file_reader::read_file_lines(path)?;
        // Create a symbol loader
        let symbol_loader;
        // Add path of the ASC file's directory to the symbol loader
        if let Some(parent) = path.parent() {
            symbol_loader = symbol::SymbolLoader::new(Some(vec![parent.to_path_buf()]));
        } else {
            symbol_loader = symbol::SymbolLoader::new(None);
        }
        Ok(Schematic::parse(lines, symbol_loader))
    }

    pub fn parse(lines: Vec<String>, mut symbol_loader: symbol::SymbolLoader) -> Schematic {
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

        for (id, line) in file_lines.enumerate() {
            let first_word = line.split_whitespace().next();
            if first_word.is_none() {
                println!(
                    "Empty line encountered: '{}', Line number: {}, skipping",
                    line,
                    id + 3
                );
                continue; // skip empty lines
            }
            let first_word = first_word.unwrap();
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
                    let last_flag = flags
                        .last_mut()
                        .expect(&format!("IOPIN line without preceding FLAG: {}", line));
                    last_flag.add_io_type(&line);
                }
                "SYMBOL" => {
                    let symbol = Symbol::from_asc_line(&line, &mut symbol_loader)
                        .expect(&format!("Failed to parse SYMBOL line: {}", &line));
                    symbols.push(symbol);
                }
                "WINDOW" | "SYMATTR" => {
                    let last_symbol = symbols.last_mut().expect(&format!(
                        "{} line without preceding SYMBOL: {}",
                        first_word, line
                    ));
                    last_symbol.add_attribute(&line);
                }
                _ => {
                    // Try to parse as shape
                    if let Some(shape) = Shape::parse_line(&line, false) {
                        shapes.push(shape);
                        continue;
                    }
                    panic!("Unknown line type: {}", line);
                }
            }
        }

        Schematic {
            sheet_size,
            wires,
            flags,
            symbols,
            shapes,
            symbol_loader: symbol_loader,
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
    fn test_basic_asc_example() {
        let document = Schematic::from_path(&PathBuf::from("test_files/text_sample.asc"))
            .expect("Failed to read ASC file");
        assert_eq!(document.wires.len(), 0);
        assert_eq!(document.flags.len(), 0);
        assert_eq!(document.symbols.len(), 0);
        assert_eq!(document.shapes.len(), 19);
    }

    #[test]
    fn test_asc_example() {
        let document = Schematic::from_path(&PathBuf::from("test_files/complex_sample.asc"))
            .expect("Failed to read ASC file");
        assert_eq!(document.wires.len(), 1);
        assert_eq!(document.flags.len(), 6);
        assert_eq!(document.symbols.len(), 12);
    }

    #[test]
    fn test_all_asc_files_from_examples() {
        let user_name = std::env::var("USER").unwrap();
        // cspell: disable-next-line
        let examples_path = format!("/home/{user}/.local/share/ltspice/dosdevices/c:/users/{user}/AppData/Local/LTspice/examples", user=user_name);
        // Recurse through all .asy files in the examples_path also in subdirectories
        let asc_files = glob::glob(&format!("{}/**/*.asc", examples_path))
            .unwrap()
            .collect::<Vec<_>>();
        let total_files = asc_files.len();

        let mut symbol_loader =
            symbol::SymbolLoader::new(Some(vec![PathBuf::from(&examples_path)]));

        println!("Found {} ASC files in LTspice lib", total_files);
        let start_time = std::time::Instant::now();
        for (current_index, entry) in asc_files.iter().enumerate() {
            let path = entry.as_ref().unwrap();
            print!(
                "\r{current_index:5} of {total_files}: Testing ASC file: {}    ",
                path.to_str().unwrap(),
                current_index = current_index + 1
            );
            let asc_file = Schematic::from_path_with_symbol_loader(path, symbol_loader)
                .expect("Failed to read ASC file");
            symbol_loader = asc_file.symbol_loader;
        }
        let duration = start_time.elapsed();
        println!();
        println!("All ASC files parsed successfully in {:?}", duration);
    }

    #[test]
    fn empty_line() {
        // cspell: disable-next-line
        let asc_file = Schematic::from_path(&PathBuf::from("/home/alexanderh/.local/share/ltspice/dosdevices/c:/users/alexanderh/AppData/Local/LTspice/examples/Applications/LT6372-1.asc"))
            .expect("Failed to read ASC file");
        asc_file.shapes.iter().for_each(|shape| {
            println!("{:?}", shape);
        });
    }
}
