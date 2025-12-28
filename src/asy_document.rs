#![allow(dead_code)]

use crate::{file_reader::read_file_lines, shape::Shape, symbol::Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    Block,
    Cell,
}

impl SymbolType {
    pub fn from_str(s: &str) -> Option<SymbolType> {
        match s.to_uppercase().as_str() {
            "BLOCK" => Some(SymbolType::Block),
            "CELL" => Some(SymbolType::Cell),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            SymbolType::Block => "BLOCK",
            SymbolType::Cell => "CELL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    position: (i32, i32),
    name: Option<String>,
    spice_order: Option<i32>,
}

impl Pin {
    pub fn from_asc_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return None;
        }

        let x = parts[1].parse::<i32>().ok()?;
        let y = parts[2].parse::<i32>().ok()?;

        let name = if parts.len() > 3 {
            Some(parts[3].to_string())
        } else {
            None
        };

        let spice_order = if parts.len() > 4 {
            parts[4].parse::<i32>().ok()
        } else {
            None
        };

        Some(Pin {
            position: (x, y),
            name,
            spice_order,
        })
    }

    pub fn add_attribute(&mut self, line: &str) -> Result<(), String> {
        // Verify the line starts with PINATTR
        if !line.starts_with("PINATTR") {
            return Err("Line does not start with PINATTR".to_string());
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err("Not enough parts in line".to_string());
        }

        match parts[1] {
            "PinName" => {
                self.name = Some(parts[2].to_string());
            }
            "SpiceOrder" => {
                if let Ok(order) = parts[2].parse::<i32>() {
                    self.spice_order = Some(order);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub struct AsyDocument {
    pub symbol: Symbol,
    pub symbol_type: SymbolType,
    pub pins: Vec<Pin>,
    pub shapes: Vec<Shape>,
}

impl AsyDocument {
    pub fn from_path(path: &str) -> Result<AsyDocument, std::io::Error> {
        let asc_content = read_file_lines(path)?;
        Ok(AsyDocument::parse(asc_content))
    }
    pub fn parse(lines: Vec<String>) -> AsyDocument {
        let mut symbol = Symbol::new();
        let mut pins: Vec<Pin> = Vec::new();
        let mut shapes: Vec<Shape> = Vec::new();

        let mut lines = lines.into_iter();

        // Remove any null characters from the first line
        let first_line = lines.next().unwrap();

        if first_line != "Version 4" {
            panic!(
                "Invalid ASY file: missing Version line: \"{}\" and should be \"Version 4\"",
                first_line
            );
        }

        // Second line should be SymbolType
        let symbol_type_line = lines.next().unwrap();
        let parts = symbol_type_line.split_whitespace().collect::<Vec<&str>>();
        if parts.len() < 2 || parts[0] != "SymbolType" {
            panic!("Invalid ASY file: missing SymbolType line");
        }
        let symbol_type = SymbolType::from_str(parts[1]).expect("Unknown SymbolType in ASY file");

        for line in lines {
            let first_word = line.split_whitespace().next().unwrap_or("");
            match first_word {
                "Version" => {
                    println!("Ignoring redundant Version line: {}", line);
                }
                "SymbolType" => {
                    println!("Ignoring redundant SymbolType line: {}", line);
                }
                "SYMATTR" | "WINDOW" => {
                    symbol.add_attribute(&line);
                }
                "PIN" => {
                    pins.push(Pin::from_asc_line(&line).expect("Failed to parse PIN line"));
                }
                "PINATTR" => {
                    if let Some(last_pin) = pins.last_mut() {
                        last_pin
                            .add_attribute(&line)
                            .expect("Failed to add PINATTR to last pin");
                    } else {
                        panic!("PINATTR found but no preceding PIN");
                    }
                }
                _ => {
                    // Check for SHAPE lines
                    if let Some(shape) = Shape::parse_line(&line, true) {
                        shapes.push(shape);
                        continue;
                    }
                    if first_word.is_empty() {
                        continue; // skip empty lines
                    }
                    println!("Unhandled line in ASY file: {}", line);
                    panic!("Unhandled line type: {}", line);
                }
            }
        }
        AsyDocument {
            symbol,
            symbol_type,
            pins,
            shapes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pin_from_asc_line() {
        let line = "PIN 100 200 Pin1 1";
        let pin = Pin::from_asc_line(line).unwrap();
        assert_eq!(pin.position, (100, 200));
        assert_eq!(pin.name, Some("Pin1".to_string()));
        assert_eq!(pin.spice_order, Some(1));
    }
    #[test]
    fn test_pin_add_attribute() {
        let mut pin = Pin {
            position: (100, 200),
            name: None,
            spice_order: None,
        };
        pin.add_attribute("PINATTR PinName PinA").unwrap();
        assert_eq!(pin.name, Some("PinA".to_string()));
        pin.add_attribute("PINATTR SpiceOrder 2").unwrap();
        assert_eq!(pin.spice_order, Some(2));
    }
    #[test]
    fn test_asy_file_from_asy_lines() {
        let lines = vec![
            "Version 4".to_string(),
            "SymbolType BLOCK".to_string(),
            "SYMATTR SomeAttribute SomeValue".to_string(),
            "PIN 100 200 Pin1 1".to_string(),
            "PINATTR PinName PinA".to_string(),
            "PINATTR SpiceOrder 2".to_string(),
        ];
        let asy_file = AsyDocument::parse(lines);
        assert_eq!(asy_file.symbol_type, SymbolType::Block);
    }

    #[test]
    fn test_asy_file_from_path() {
        let asy_file =
            AsyDocument::from_path("test_files/Demo.asy").expect("Failed to read ASY file");
        assert_eq!(asy_file.symbol_type, SymbolType::Block);
    }

    #[test]
    fn test_all_asy_files_from_lib() {
        let user_name = std::env::var("USER").unwrap();
        // cspell: disable-next-line
        let lib_path = format!("/home/{user}/.local/share/ltspice/dosdevices/c:/users/{user}/AppData/Local/LTspice/lib/sym", user=user_name);
        // Recurse through all .asy files in the lib_path also in subdirectories
        let asy_files = glob::glob(&format!("{}/**/*.asy", lib_path))
            .unwrap()
            .collect::<Vec<_>>();
        let total_files = asy_files.len();
        println!("Found {} ASY files in LTspice lib", total_files);
        let start_time = std::time::Instant::now();
        for (current_index, entry) in asy_files.iter().enumerate() {
            let path = entry.as_ref().unwrap();
            print!(
                "\r{current_index:5} of {total_files}: Testing ASY file: {}    ",
                path.to_str().unwrap(),
                current_index = current_index + 1
            );
            let _asy_file =
                AsyDocument::from_path(path.to_str().unwrap()).expect("Failed to read ASY file");
        }
        let duration = start_time.elapsed();
        println!();
        println!("All ASY files parsed successfully in {:?}", duration);
    }
}
