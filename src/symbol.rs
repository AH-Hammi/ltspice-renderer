//! All struct needed to represent an LTspice symbol
//!
//! Is able to parse full .asy files,
//! and extract symbol information from .asc files.
//!
//! This is done because both can contain the "SYMATTR" definitions
//! as well as the windowing information for the symbol.

#![allow(dead_code)]

use core::panic;
use std::{collections::HashMap, path::PathBuf};

use crate::{file_reader::read_file_lines, shape::Shape};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pin {
    position: (i32, i32),
    name: Option<String>,
    spice_order: Option<i32>,
}

impl Pin {
    pub fn from_line(line: &str) -> Option<Self> {
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
    used_symbols: HashMap<String, LibrarySymbol>,
}

impl SymbolLoader {
    fn default_library_paths() -> Vec<PathBuf> {
        let user_name = whoami::username().unwrap();
        // Determine on which OS we are running
        if cfg!(target_os = "windows") {
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
            // LTspice on Linux via Wine
            return vec![
                PathBuf::from(
                    // cspell: disable-next-line
                    format!("/home/{user}/.wine/drive_c/users/{user}/Documents/LTspice", user=user_name),
                ),
                PathBuf::from(
                    // cspell: disable-next-line
                    format!("/home/{user}/.wine/dosdevices/c:/users/{user}/AppData/Local/LTspice/lib/sym", 
                    user=user_name),
                ),
            ];
        }
        panic!(
            "Unsupported OS for default LTspice library paths {}",
            std::env::consts::OS
        );
    }

    pub fn new(extra_library_paths: Option<Vec<PathBuf>>) -> Self {
        let mut library_paths = extra_library_paths.unwrap_or_else(Vec::new);
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
        let symbol_name = symbol_name.split('/').last().unwrap();
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
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Symbol not found: {}", symbol_name),
        ));
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolRotation {
    R0,
    R90,
    R180,
    R270,
    M0,
    M90,
    M180,
    M270,
}

impl SymbolRotation {
    pub fn from_str(s: &str) -> Option<SymbolRotation> {
        match s {
            "R0" => Some(SymbolRotation::R0),
            "R90" => Some(SymbolRotation::R90),
            "R180" => Some(SymbolRotation::R180),
            "R270" => Some(SymbolRotation::R270),
            "M0" => Some(SymbolRotation::M0),
            "M90" => Some(SymbolRotation::M90),
            "M180" => Some(SymbolRotation::M180),
            "M270" => Some(SymbolRotation::M270),
            _ => None,
        }
    }
}

impl std::fmt::Display for SymbolRotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SymbolRotation::R0 => "0",
            SymbolRotation::R90 => "90",
            SymbolRotation::R180 => "180",
            SymbolRotation::R270 => "270",
            SymbolRotation::M0 => "0",
            SymbolRotation::M90 => "90",
            SymbolRotation::M180 => "180",
            SymbolRotation::M270 => "270",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub symbol_id: String,
    pub position: (i32, i32),
    pub rotation: SymbolRotation,
}

impl Symbol {
    pub fn from_line(line: &str, symbol_loader: &mut SymbolLoader) -> Result<Symbol, String> {
        // The format is SYMBOL {Type} {PositionX} {PositionY} {Rotation} ...
        let parts = line.splitn(6, ' ').collect::<Vec<&str>>();
        if parts.len() < 5 || parts[0] != "SYMBOL" {
            return Err(format!("Invalid SYMBOL line: {}", line));
        }
        let symbol_type = parts[1].to_string();
        let symbol_id = symbol_loader
            .load_symbol(&symbol_type)
            .map_err(|e| e.to_string())?;
        let position_x = parts[2].parse::<i32>().map_err(|e| e.to_string())?;
        let position_y = parts[3].parse::<i32>().map_err(|e| e.to_string())?;
        let rotation = SymbolRotation::from_str(parts[4])
            .ok_or_else(|| format!("Invalid rotation: {}", parts[4]))?;
        Ok(Symbol {
            symbol_id,
            position: (position_x, position_y),
            rotation,
        })
    }
    pub fn add_symbol_attribute(&mut self, _line: &str) {
        if !_line.starts_with("SYMATTR") {
            panic!("Line is not a SYMATTR line: {}", _line);
        }
        let parts = _line.splitn(3, ' ').collect::<Vec<&str>>();
        if parts.len() < 3 {
            panic!("Invalid SYMATTR line: {}", _line);
        }
        let _attr_name = parts[1];
        let _attr_value = parts[2];
    }

    pub fn add_window_attribute(&mut self, _line: &str) {
        if !_line.starts_with("WINDOW") {
            panic!("Line is not a WINDOW line: {}", _line);
        }
        let parts = _line.splitn(4, ' ').collect::<Vec<&str>>();
        if parts.len() < 4 {
            panic!("Invalid WINDOW line: {}", _line);
        }
        let _window_type = parts[1];
        let _window_value = parts[2];
    }

    pub fn add_attribute(&mut self, _line: &str) {
        let parts = _line.splitn(5, ' ').collect::<Vec<&str>>();
        match parts[0] {
            "WINDOW" => {
                self.add_window_attribute(_line);
            }
            "SYMATTR" => {
                self.add_symbol_attribute(_line);
            }
            _ => {
                panic!("Unhandled symbol attribute line: {}", _line);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_from_line() {
        let line = "PIN 100 200 Pin1 1";
        let pin = Pin::from_line(line).unwrap();
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
            available_symbols.len() > 0,
            "No available symbols found in default library paths"
        );
        // Mark all symbols as used
        for (id, symbol_name) in available_symbols.keys().enumerate() {
            print!(
                "\rLoading symbol {}/{}: {}",
                id + 1,
                available_symbols.len(),
                symbol_name
            );
            let _ = symbol_loader.load_symbol(symbol_name);
        }
        assert_eq!(symbol_loader.used_symbols.len(), available_symbols.len());
    }

    #[test]
    fn test_symbol_from_asc() {
        let line = "SYMBOL res 100 200 R0";
        let mut symbol_loader = SymbolLoader::new(None);
        let symbol = Symbol::from_line(line, &mut symbol_loader).unwrap();
        assert_eq!(symbol.position, (100, 200));
        assert_eq!(symbol.rotation, SymbolRotation::R0);
    }
}
