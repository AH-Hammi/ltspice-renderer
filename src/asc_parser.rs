use crate::shape_parser::Shape;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextJustification {
    Left,
    Center,
    Right,
    Top,
    Bottom,
    VLeft,
    VCenter,
    VRight,
    VTop,
    VBottom,
    Invisible,
}

impl TextJustification {
    pub fn from_str(s: &str) -> Option<TextJustification> {
        match s {
            "Left" => Some(TextJustification::Left),
            "Center" => Some(TextJustification::Center),
            "Right" => Some(TextJustification::Right),
            "Top" => Some(TextJustification::Top),
            "Bottom" => Some(TextJustification::Bottom),
            "VLeft" => Some(TextJustification::VLeft),
            "VCenter" => Some(TextJustification::VCenter),
            "VRight" => Some(TextJustification::VRight),
            "VTop" => Some(TextJustification::VTop),
            "VBottom" => Some(TextJustification::VBottom),
            "Invisible" => Some(TextJustification::Invisible),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextSize {
    SIZE0625,
    SIZE10,
    SIZE15,
    SIZE20,
    SIZE25,
    SIZE35,
    SIZE50,
    SIZE70,
}

impl TextSize {
    pub fn from_str(s: &str) -> Option<TextSize> {
        match s {
            "0" => Some(TextSize::SIZE0625),
            "1" => Some(TextSize::SIZE10),
            "2" => Some(TextSize::SIZE15),
            "3" => Some(TextSize::SIZE20),
            "4" => Some(TextSize::SIZE25),
            "5" => Some(TextSize::SIZE35),
            "6" => Some(TextSize::SIZE50),
            "7" => Some(TextSize::SIZE70),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextType {
    SpiceDirective,
    Comment,
}

impl TextType {
    pub fn from_str(s: &str) -> Option<TextType> {
        match s {
            "!" => Some(TextType::SpiceDirective),
            ";" => Some(TextType::Comment),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    position: (i32, i32),
    justification: TextJustification,
    size: TextSize,
    text_type: TextType,
    content: String,
}

impl Text {
    fn from_asc_line(line: &str) -> Option<Text> {
        // The format is TEXT {PositionX} {PositionY} {Justification} {Size} {TypeSpecifier}{Content}
        let parts = line.splitn(6, ' ').collect::<Vec<&str>>();
        if parts.len() < 6 || parts[0] != "TEXT" {
            return None;
        }
        let position_x = parts[1].parse::<i32>().ok()?;
        let position_y = parts[2].parse::<i32>().ok()?;
        let justification = TextJustification::from_str(parts[3])?;
        let size = TextSize::from_str(parts[4])?;
        let text_type = TextType::from_str(&parts[5][..1])?;
        let content = &parts[5][1..];
        Some(Text {
            position: (position_x, position_y),
            justification,
            size,
            text_type,
            content: content.to_string(),
        })
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    symbol_type: String,
    position: (i32, i32),
    rotation: SymbolRotation,
}

impl Symbol {
    fn from_asc_line(line: &str) -> Option<Symbol> {
        // The format is SYMBOL {Type} {PositionX} {PositionY} {Rotation} ...
        let parts = line.splitn(6, ' ').collect::<Vec<&str>>();
        if parts.len() < 5 || parts[0] != "SYMBOL" {
            return None;
        }
        let symbol_type = parts[1].to_string();
        let position_x = parts[2].parse::<i32>().ok()?;
        let position_y = parts[3].parse::<i32>().ok()?;
        let rotation = SymbolRotation::from_str(parts[4])?;
        Some(Symbol {
            symbol_type,
            position: (position_x, position_y),
            rotation,
        })
    }
    fn add_attribute(&mut self, _line: &str) {
        let parts = _line.splitn(5, ' ').collect::<Vec<&str>>();
        match parts[0] {
            "WINDOW" => {
                // Handle WINDOW attributes if needed
            }
            "SYMATTR" => {
                // Handle SYMATTR attributes if needed
            }
            _ => {
                panic!("Internal error")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcsDocument {
    sheet_size: (i32, i32),
    texts: Vec<Text>,
    wires: Vec<Wire>,
    flags: Vec<Flag>,
    symbols: Vec<Symbol>,
    shapes: Vec<Shape>,
}

impl AcsDocument {
    fn check_version_line(line: &str) {
        if !line.starts_with("Version ") {
            panic!("Invalid ASC file: missing Version line");
        }
        if line.trim() != "Version 4.1" {
            panic!("Unsupported ASC version: {}", line);
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

    pub fn parse(asc_content: &str) -> AcsDocument {
        let mut texts = Vec::new();
        let mut wires = Vec::new();
        let mut flags = Vec::new();
        let mut symbols = Vec::new();
        let mut shapes = Vec::new();

        let mut file_lines = asc_content.lines().into_iter();

        Self::check_version_line(file_lines.next().unwrap());

        let sheet_size =
            Self::parse_sheet_line(file_lines.next().expect("Couldn't read second line"));

        for line in file_lines {
            let parts = line.splitn(5, ' ').collect::<Vec<&str>>();
            // The first line contains the Version
            if parts[0] == "Version" {
                continue;
            }
            match parts[0] {
                "TEXT" => {
                    if let Some(text) = Text::from_asc_line(line) {
                        texts.push(text);
                    } else {
                        panic!("Failed to parse TEXT line: {}", line);
                    }
                }
                "WIRE" => {
                    if let Some(wire) = Wire::from_asc_line(line) {
                        wires.push(wire);
                    } else {
                        panic!("Failed to parse WIRE line: {}", line);
                    }
                }
                "FLAG" => {
                    if let Some(flag) = Flag::from_asc_line(line) {
                        flags.push(flag);
                    } else {
                        panic!("Failed to parse FLAG line: {}", line);
                    }
                }
                "IOPIN" => {
                    if let Some(last_flag) = flags.last_mut() {
                        last_flag.add_io_type(line);
                    } else {
                        panic!("IOPIN line without preceding FLAG: {}", line);
                    }
                }
                "SYMBOL" => {
                    if let Some(symbol) = Symbol::from_asc_line(line) {
                        symbols.push(symbol);
                    } else {
                        panic!("Failed to parse SYMBOL line: {}", line);
                    }
                }
                "WINDOW" | "SYMATTR" => {
                    if let Some(last_symbol) = symbols.last_mut() {
                        last_symbol.add_attribute(line);
                    } else {
                        panic!("WINDOW line without preceding SYMBOL: {}", line);
                    }
                }
                _ => {
                    // Try to parse as shape
                    if let Some(shape) = Shape::from_asc_line(line) {
                        shapes.push(shape);
                    } else {
                        panic!("Unknown line type: {}", line);
                    }
                }
            }
        }

        AcsDocument {
            sheet_size,
            texts,
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
    fn test_parse_text() {
        let line = "TEXT 480 720 Center 2 ;This is a comment";
        let text = Text::from_asc_line(line).expect("Failed to parse TEXT line");
        assert_eq!(text.position, (480, 720));
        assert_eq!(text.justification, TextJustification::Center);
        assert_eq!(text.size, TextSize::SIZE15);
        assert_eq!(text.text_type, TextType::Comment);
        assert_eq!(text.content, "This is a comment");
    }
    #[test]
    fn test_parse_wire() {
        let line = "WIRE 480 720 640 720";
        let wire = Wire::from_asc_line(line).expect("Failed to parse WIRE line");
        assert_eq!(wire.start, (480, 720));
        assert_eq!(wire.end, (640, 720));
    }
    #[test]
    fn test_parse_flag() {
        let line = "FLAG 640 720 Vout";
        let flag = Flag::from_asc_line(line).expect("Failed to parse FLAG line");
        assert_eq!(flag.position, (640, 720));
        assert_eq!(flag.name, "Vout");
        assert_eq!(flag.io_type, None);
    }

    #[test]
    fn test_asc_example() {
        let asc_content = r#"Version 4.1
SHEET 1 1728 1704
WIRE 416 1200 32 1200
FLAG 336 1520 0
FLAG 336 1296 PortTypeNone
FLAG 336 1328 Input
IOPIN 336 1328 In
FLAG 336 1376 Output
IOPIN 336 1376 Out
FLAG 336 1424 BiDir
IOPIN 336 1424 BiDir
FLAG 336 1472 COM
SYMBOL Hierachical 976 784 R0
WINDOW 0 -22 -62 Bottom 2
WINDOW 39 -74 120 Left 1
SYMATTR InstName X1
SYMATTR SpiceLine test ;Hier steht ein Kommentar
SYMBOL voltage 48 1264 R0
SYMATTR InstName V1
SYMBOL res 32 1408 R0
SYMATTR InstName R1
SYMBOL cap 32 1552 R0
SYMATTR InstName C1
TEXT 160 656 Left 2 ;Left
TEXT 0 656 Right 2 ;Right
TEXT 80 656 Center 2 ;Center
TEXT 80 688 Top 2 ;Top
TEXT 80 632 Bottom 2 ;Bottom
TEXT 0 760 Left 2 ;Not Visible
TEXT 64 776 Left 0 ;0.625
TEXT 64 792 Left 1 ;1.0
TEXT 64 808 Left 2 ;1.5
TEXT 64 840 Left 3 ;2.0
TEXT 64 872 Left 4 ;2.5
TEXT 64 920 Left 5 ;3.5
TEXT 64 984 Left 6 ;5.0
TEXT 64 1064 Left 7 ;7.0
TEXT 584 656 Left 2 !Spice Directive
TEXT 472 608 VLeft 2 ;Vertical Left
TEXT 472 840 VRight 2 ;Vertical Right
TEXT 472 728 VCenter 2 ;Vertical Center
TEXT 496 720 VTop 2 ;Vertical Top
TEXT 432 720 VBottom 2 ;Vertical Bottom
"#;
        let document = AcsDocument::parse(asc_content);
        println!("Parsed document: {:?}", document);
        assert_eq!(document.sheet_size, (1728, 1704));
        assert_eq!(document.texts.len(), 20);
        assert_eq!(document.wires.len(), 1);
        assert_eq!(document.flags.len(), 6);
        assert_eq!(document.symbols.len(), 4);
    }
}
