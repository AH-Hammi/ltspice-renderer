//! All struct needed to represent an LTspice symbol
//!
//! Is able to parse full .asy files,
//! and extract symbol information from .asc files.
//!
//! This is done because both can contain the "SYMATTR" definitions
//! as well as the windowing information for the symbol.

use crate::symbol_loader::SymbolLoader;

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
    fn test_symbol_from_asc() {
        let line = "SYMBOL res 100 200 R0";
        let mut symbol_loader = SymbolLoader::new(None);
        let symbol = Symbol::from_line(line, &mut symbol_loader).unwrap();
        assert_eq!(symbol.position, (100, 200));
        assert_eq!(symbol.rotation, SymbolRotation::R0);
    }
}
