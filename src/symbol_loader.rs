#![allow(dead_code)]

use std::{collections::HashMap, path::PathBuf};

use crate::{
    file_reader::read_file_lines,
    shape::{Shape, Text},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// A pin in a symbol is equivalent to a text element. With the addition being the spice order of the pin.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pin {
    pub text: Text,
    pub spice_order: Option<i32>,
}

impl Pin {
    pub fn from_line(line: &str) -> Result<Self, &str> {
        // PIN {X} {Y} {JUSTIFICATION} {OFFSET}
        Ok(Pin {
            text: Text::parse_pin_line(line)?,
            spice_order: None,
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
                self.text.content = parts[2].to_string();
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibrarySymbol {
    pub symbol_type: SymbolType,
    pub pins: Vec<Pin>,
    pub shapes: Vec<Shape>,
    pub attributes: HashMap<String, String>,
    pub window_attributes: HashMap<String, String>,
}

impl LibrarySymbol {
    pub fn from_path(path: &PathBuf) -> Result<LibrarySymbol, std::io::Error> {
        let content = read_file_lines(path)?;
        Ok(LibrarySymbol::parse(content))
    }
    pub fn parse(lines: Vec<String>) -> LibrarySymbol {
        let mut pins: Vec<Pin> = Vec::new();
        let mut shapes: Vec<Shape> = Vec::new();

        let mut lines = lines.into_iter();

        let mut attributes: HashMap<String, String> = HashMap::new();
        let mut window_attributes: HashMap<String, String> = HashMap::new();

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
                "SYMATTR" => {
                    let parts = line.splitn(3, ' ').collect::<Vec<&str>>();
                    if parts.len() < 3 {
                        panic!("Invalid SYMATTR line: {}", line);
                    }
                    attributes.insert(parts[1].to_string(), parts[2].to_string());
                }
                "WINDOW" => {
                    let parts = line.splitn(4, ' ').collect::<Vec<&str>>();
                    if parts.len() < 4 {
                        panic!("Invalid WINDOW line: {}", line);
                    }
                    window_attributes.insert(parts[1].to_string(), parts[2].to_string());
                }
                "PIN" => {
                    pins.push(Pin::from_line(&line).expect("Failed to parse PIN line"));
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
        LibrarySymbol {
            symbol_type,
            pins,
            shapes,
            attributes,
            window_attributes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolLoader {
    available_symbols: HashMap<String, PathBuf>,
    short_name_to_id: HashMap<String, String>,
    pub(crate) used_symbols: HashMap<String, LibrarySymbol>,
}

impl SymbolLoader {
    fn default_library_paths() -> Vec<PathBuf> {
        // Determine on which OS we are running
        if cfg!(target_os = "windows") {
            let user_name = whoami::username().unwrap();
            return vec![
                PathBuf::from(format!(
                    "C:\\Users\\{user}\\AppData\\Local\\LTspice\\lib\\sym",
                    user = user_name
                )),
                PathBuf::from(format!(
                    "C:\\Users\\{user}\\Documents\\LTspice\\lib\\sym",
                    user = user_name
                )),
            ];
        } else if cfg!(target_os = "macos") {
            panic!("MacOS LTspice library path not implemented yet");
        } else if cfg!(target_os = "linux") {
            let user_name = std::env::var("USER").unwrap();
            // LTspice on Linux via Wine
            let mut local_share =
                PathBuf::from(format!("/home/{user}/.local/share", user = user_name));
            let wineprefixes = local_share.join("wineprefixes");
            if wineprefixes.exists() {
                local_share = wineprefixes;
            }
            return vec![
                local_share.join(format!(
                    "ltspice/drive_c/users/{user}/Documents/LTspice/lib/sym",
                    user = user_name
                )),
                local_share.join(format!(
                    "ltspice/dosdevices/c:/users/{user}/AppData/Local/LTspice/lib/sym",
                    user = user_name
                )),
            ];
        }
        panic!(
            "Unsupported OS for default LTspice library paths {}",
            std::env::consts::OS
        );
    }

    pub fn new(extra_library_paths: Option<Vec<PathBuf>>) -> Self {
        let mut library_paths = extra_library_paths.unwrap_or_default();
        library_paths.extend(Self::default_library_paths());

        let (available_symbols, name_to_full_name) = Self::load_available_symbols(library_paths);

        SymbolLoader {
            available_symbols,
            short_name_to_id: name_to_full_name,
            used_symbols: HashMap::new(),
        }
    }

    fn load_available_symbols(
        library_paths: Vec<PathBuf>,
    ) -> (HashMap<String, PathBuf>, HashMap<String, String>) {
        // Load all available .asy file paths
        let mut available_symbols: HashMap<String, PathBuf> = HashMap::new();
        let mut short_name_to_id: HashMap<String, String> = HashMap::new();
        for lib_path in library_paths.iter().rev() {
            let asy_files = glob::glob(&format!("{}/**/*.asy", lib_path.display()))
                .unwrap()
                .collect::<Vec<_>>();
            for entry in asy_files {
                if entry.is_err() {
                    println!("Failed to read ASY file entry: {:?}", entry);
                    continue;
                }
                let path = entry.unwrap();
                let key = path.strip_prefix(lib_path);
                if key.is_err() {
                    println!("Failed to get key for symbol from path: {}", path.display());
                    continue;
                }
                let key = key
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .strip_suffix(".asy")
                    .unwrap()
                    .to_lowercase();
                if available_symbols.contains_key(&key) {
                    println!("Warning: Duplicate symbol key found: {}", key);
                }
                let short_name = path.file_stem().unwrap().to_str().unwrap().to_lowercase();
                available_symbols.insert(key.clone(), path);
                // Also map short name to full key
                short_name_to_id.insert(short_name, key);
            }
        }
        println!("Loaded {} symbols into cache", available_symbols.len());
        (available_symbols, short_name_to_id)
    }

    /// Check if a symbol is available and return its full path if it is.
    fn check_symbol_availability(&mut self, symbol_name: &str) -> Result<String, std::io::Error> {
        let symbol_name = symbol_name
            .replace(r"AutoGenerated\\", "")
            .replace(r"\\", "/")
            .to_lowercase();
        if self.available_symbols.contains_key(&symbol_name) {
            return Ok(symbol_name);
        }
        let symbol_name = symbol_name.split('/').next_back().unwrap();
        if self.short_name_to_id.contains_key(symbol_name) {
            return Ok(self.short_name_to_id[symbol_name].clone());
        }
        // Dump available symbols to file for debugging
        std::fs::write(
            "available_symbols.txt",
            self.available_symbols
                .keys()
                .cloned()
                .collect::<Vec<String>>()
                .join("\n"),
        )
        .unwrap();
        std::eprintln!("Available symbols written to available_symbols.txt for debugging.");
        std::eprintln!("Requested symbol: {}", symbol_name);
        std::fs::write(
            "name_to_id_map.txt",
            self.short_name_to_id
                .iter()
                .map(|(k, v)| format!("{} -> {}", k, v))
                .collect::<Vec<String>>()
                .join("\n"),
        )
        .unwrap();
        std::eprintln!("Name to ID map written to name_to_id_map.txt for debugging.");
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Symbol not found: {}", symbol_name),
        ))
    }

    /// Mark a symbol as being used. This will add the symbol to the used_symbols list
    /// and return a unique ID for the symbol instance.
    pub fn load_symbol(&mut self, symbol_name: &str) -> Result<String, std::io::Error> {
        let symbol_id = self.check_symbol_availability(symbol_name);
        match symbol_id {
            Err(e) => Err(e),
            Ok(symbol_id) => {
                self.used_symbols.insert(
                    symbol_id.clone(),
                    LibrarySymbol::from_path(&self.available_symbols[&symbol_id])?,
                );
                Ok(symbol_id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::shape::{Point, TextJustification};

    use super::*;

    #[test]
    fn test_pin_from_line() {
        let line = "PIN -128 0 Left 8";
        let pin = Pin::from_line(line).unwrap();
        assert_eq!(pin.text.position, Point { x: -128, y: 0 });
        assert_eq!(pin.text.justification, TextJustification::Left);
        assert_eq!(pin.text.offset, 8)
    }

    #[test]
    fn test_pin_add_attribute() {
        let mut pin = Pin {
            text: Text {
                position: Point { x: 100, y: 200 },
                content: String::new(),
                justification: TextJustification::Left,
                vertical: false,
                offset: 8,
                size: crate::shape::TextSize::SIZE15,
                text_type: None,
            },
            spice_order: None,
        };
        pin.add_attribute("PINATTR PinName PinA").unwrap();
        assert_eq!(pin.text.content, "PinA");
        pin.add_attribute("PINATTR SpiceOrder 2").unwrap();
        assert_eq!(pin.spice_order, Some(2));
    }

    #[test]
    fn test_asy_file_from_asy_lines() {
        let lines = vec![
            "Version 4".to_string(),
            "SymbolType BLOCK".to_string(),
            "SYMATTR SomeAttribute SomeValue".to_string(),
            "PIN 100 200 Left 20".to_string(),
            "PINATTR PinName PinA".to_string(),
            "PINATTR SpiceOrder 2".to_string(),
        ];
        let asy_file = LibrarySymbol::parse(lines);
        assert_eq!(asy_file.symbol_type, SymbolType::Block);
    }

    #[test]
    fn test_asy_file_from_path() {
        let asy_file = LibrarySymbol::from_path(&PathBuf::from("test_files/Demo.asy"))
            .expect("Failed to read ASY file");
        assert_eq!(asy_file.symbol_type, SymbolType::Block);
    }

    #[test]
    fn load_all_available_library_symbols() {
        let mut symbol_loader = SymbolLoader::new(None);
        let available_symbols = symbol_loader.available_symbols.clone();
        assert!(
            !available_symbols.is_empty(),
            "No available symbols found in default library paths"
        );
        // Mark all symbols as used
        for (id, symbol_name) in available_symbols.keys().enumerate() {
            print!(
                "\rLoading symbol {}/{}: {}                                                            ",
                id + 1,
                available_symbols.len(),
                symbol_name
            );
            let _ = symbol_loader.load_symbol(symbol_name);
        }
        println!();
        println!("All symbols loaded successfully!");
        assert_eq!(symbol_loader.used_symbols.len(), available_symbols.len());
    }
}
