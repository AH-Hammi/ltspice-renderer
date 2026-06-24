#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
    DashDotted,
    DashDotDotted,
}

impl LineStyle {
    fn from_code(code: &str) -> LineStyle {
        match code {
            "1" => LineStyle::Dashed,
            "2" => LineStyle::Dotted,
            "3" => LineStyle::DashDotted,
            "4" => LineStyle::DashDotDotted,
            _ => LineStyle::Solid,
        }
    }
    pub fn to_svg_dasharray(self) -> &'static str {
        match self {
            LineStyle::Solid => "none",
            LineStyle::Dashed => "4,2",
            LineStyle::Dotted => "1,2",
            LineStyle::DashDotted => "4,2,1,2",
            LineStyle::DashDotDotted => "4,2,1,2,1,2",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Line {
    pub(crate) start: Point,
    pub(crate) end: Point,
    pub(crate) style: LineStyle,
}

impl Line {
    pub fn parse_line(line: &str) -> Option<Line> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            return None;
        }
        if parts[0] != "LINE" {
            return None;
        }
        if parts[1] != "Normal" {
            return None;
        }
        let start = Point {
            x: parts[2].parse::<i32>().ok()?,
            y: parts[3].parse::<i32>().ok()?,
        };
        let end = Point {
            x: parts[4].parse::<i32>().ok()?,
            y: parts[5].parse::<i32>().ok()?,
        };
        let style = if parts.len() > 6 {
            LineStyle::from_code(parts[6])
        } else {
            LineStyle::Solid
        };
        if end < start {
            return Some(Line { end, start, style });
        }
        Some(Line { start, end, style })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rectangle {
    pub(crate) top_left: Point,
    pub(crate) bottom_right: Point,
    pub(crate) style: LineStyle,
}

impl Rectangle {
    pub fn parse_line(line: &str) -> Result<Rectangle, &'static str> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            return Err("Not enough parts for RECTANGLE");
        }
        if parts[0] != "RECTANGLE" {
            return Err("Not a RECTANGLE line");
        }
        if parts[1] != "Normal" {
            return Err("Not a Normal RECTANGLE line");
        }
        let first = Point {
            x: parts[2].parse::<i32>().map_err(|_| "Invalid top left x")?,
            y: parts[3].parse::<i32>().map_err(|_| "Invalid top left y")?,
        };
        let second = Point {
            x: parts[4]
                .parse::<i32>()
                .map_err(|_| "Invalid bottom right x")?,
            y: parts[5]
                .parse::<i32>()
                .map_err(|_| "Invalid bottom right y")?,
        };
        let top_left = Point {
            x: first.x.min(second.x),
            y: first.y.min(second.y),
        };
        let bottom_right = Point {
            x: first.x.max(second.x),
            y: first.y.max(second.y),
        };
        let style = if parts.len() > 6 {
            LineStyle::from_code(parts[6])
        } else {
            LineStyle::Solid
        };
        Ok(Rectangle {
            top_left,
            bottom_right,
            style,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Circle {
    pub(crate) top_left: Point,
    pub(crate) bottom_right: Point,
    pub(crate) style: LineStyle,
}
impl Circle {
    pub fn parse_line(line: &str) -> Option<Circle> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            return None;
        }
        if parts[0] != "CIRCLE" {
            return None;
        }
        if parts[1] != "Normal" {
            return None;
        }
        let mut top_left_x = parts[2].parse::<i32>().ok()?;
        let mut top_left_y = parts[3].parse::<i32>().ok()?;
        let mut bottom_right_x = parts[4].parse::<i32>().ok()?;
        let mut bottom_right_y = parts[5].parse::<i32>().ok()?;
        if top_left_x < bottom_right_x {
            std::mem::swap(&mut top_left_x, &mut bottom_right_x);
        }
        if top_left_y < bottom_right_y {
            std::mem::swap(&mut top_left_y, &mut bottom_right_y);
        }
        let style = if parts.len() > 6 {
            LineStyle::from_code(parts[6])
        } else {
            LineStyle::Solid
        };
        Some(Circle {
            top_left: Point {
                x: top_left_x,
                y: top_left_y,
            },
            bottom_right: Point {
                x: bottom_right_x,
                y: bottom_right_y,
            },
            style,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Arc {
    pub(crate) top_left: Point,
    pub(crate) bottom_right: Point,
    pub(crate) end: Point,
    pub(crate) start: Point,
    pub(crate) style: LineStyle,
}

impl Arc {
    pub fn parse_line(line: &str) -> Result<Arc, &'static str> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            return Err("Not enough parts for ARC");
        }
        if parts[0] != "ARC" {
            return Err("Not an ARC line");
        }
        if parts[1] != "Normal" {
            return Err("Not a Normal ARC line");
        }
        let top_left = Point {
            x: parts[2].parse::<i32>().map_err(|_| "Invalid top left x")?,
            y: parts[3].parse::<i32>().map_err(|_| "Invalid top left y")?,
        };
        let bottom_right = Point {
            x: parts[4]
                .parse::<i32>()
                .map_err(|_| "Invalid bottom right x")?,
            y: parts[5]
                .parse::<i32>()
                .map_err(|_| "Invalid bottom right y")?,
        };
        let end = Point {
            x: parts[6].parse::<i32>().map_err(|_| "Invalid end x")?,
            y: parts[7].parse::<i32>().map_err(|_| "Invalid end y")?,
        };
        let start = Point {
            x: parts[8].parse::<i32>().map_err(|_| "Invalid start x")?,
            y: parts[9].parse::<i32>().map_err(|_| "Invalid start y")?,
        };
        let style = if parts.len() > 10 {
            LineStyle::from_code(parts[10])
        } else {
            LineStyle::Solid
        };
        Ok(Arc {
            top_left,
            bottom_right,
            end,
            start,
            style,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextJustification {
    Left,
    Center,
    Right,
    Top,
    Bottom,
    Invisible,
}

impl TextJustification {
    pub fn from_str(s: &str) -> Result<Self, &str> {
        match s.to_lowercase().as_str() {
            "left" => Ok(TextJustification::Left),
            "center" => Ok(TextJustification::Center),
            "right" => Ok(TextJustification::Right),
            "top" => Ok(TextJustification::Top),
            "bottom" => Ok(TextJustification::Bottom),
            "invisible" | "none" => Ok(TextJustification::Invisible),
            _ => Err("Invalid text justification"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn from_str(s: &str) -> Result<Self, &str> {
        match s {
            "0" => Ok(TextSize::SIZE0625),
            "1" => Ok(TextSize::SIZE10),
            "2" => Ok(TextSize::SIZE15),
            "3" => Ok(TextSize::SIZE20),
            "4" => Ok(TextSize::SIZE25),
            "5" => Ok(TextSize::SIZE35),
            "6" => Ok(TextSize::SIZE50),
            "7" => Ok(TextSize::SIZE70),
            _ => Err("Invalid text size"),
        }
    }
    pub fn multiplier(&self) -> f32 {
        match self {
            TextSize::SIZE0625 => 0.625,
            TextSize::SIZE10 => 1.0,
            TextSize::SIZE15 => 1.5,
            TextSize::SIZE20 => 2.0,
            TextSize::SIZE25 => 2.5,
            TextSize::SIZE35 => 3.5,
            TextSize::SIZE50 => 5.0,
            TextSize::SIZE70 => 7.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextType {
    SpiceDirective,
    Comment,
}

impl TextType {
    fn from_str(s: &str) -> Option<TextType> {
        let first_char = s.chars().next()?;
        match first_char {
            '!' => Some(TextType::SpiceDirective),
            ';' => Some(TextType::Comment),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Text {
    pub(crate) position: Point,
    pub(crate) justification: TextJustification,
    pub(crate) vertical: bool,
    pub(crate) size: TextSize,
    pub(crate) text_type: Option<TextType>,
    pub(crate) content: String,
    pub(crate) offset: i32,
}

impl Text {
    fn parse_position_and_justification(
        parts: Vec<&str>,
    ) -> Result<(Point, TextJustification, bool), &str> {
        let position = Point {
            x: parts[0]
                .parse::<i32>()
                .expect("Couldn't parse first number"),
            y: parts[1]
                .parse::<i32>()
                .expect("Couldn't parse second number"),
        };

        // If first character of justification is 'V', it's vertical
        let vertical = parts[2].starts_with('V');
        // Remove 'V' if present to get actual justification
        let justification_str = if vertical { &parts[2][1..] } else { parts[2] };
        let justification = TextJustification::from_str(justification_str)?;
        Ok((position, justification, vertical))
    }

    pub fn parse_pin_line(line: &str) -> Result<Self, &str> {
        if !line.starts_with("PIN") {
            return Err("Line does not start with \"PIN\"");
        }
        let parts = line.splitn(5, ' ').collect::<Vec<&str>>();
        if parts.len() < 5 {
            return Err("Invalid line format");
        }
        let (position, justification, vertical) =
            Text::parse_position_and_justification(parts[1..4].to_vec())?;
        let offset = parts[4]
            .parse::<i32>()
            .expect("Couldn't parse second number");
        Ok(Text {
            position,
            justification,
            vertical,
            size: TextSize::SIZE15,
            text_type: None,
            content: "".to_string(),
            offset,
        })
    }

    pub fn parse_text_line(line: &str, is_symbol: bool) -> Result<Self, &str> {
        // The format is TEXT {PositionX} {PositionY} {Justification} {Size} {TypeSpecifier}{Content}
        if !line.starts_with("TEXT") {
            return Err("Line does not start with \"TEXT\"");
        }
        let parts = line.splitn(6, ' ').collect::<Vec<&str>>();
        if parts.len() < 6 {
            return Err("Invalid line format");
        }
        let (position, justification, vertical) =
            Text::parse_position_and_justification(parts[1..4].to_vec())?;

        let size = TextSize::from_str(parts[4])?;

        let (text_type, content): (Option<TextType>, String) = if is_symbol {
            // For symbols, default to no type specifier
            (None, parts[5].to_string())
        } else {
            // For non-symbols, check for type specifier
            (
                Some(TextType::from_str(parts[5]).unwrap()),
                parts[5][1..].to_string(),
            )
        };
        Ok(Text {
            position,
            justification,
            vertical,
            size,
            text_type,
            content,
            offset: 0,
        })
    }
}

// present all shapes as a single enum
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Shape {
    Line(Line),
    Rectangle(Rectangle),
    Circle(Circle),
    Arc(Arc),
    Text(Text),
}

impl Shape {
    pub fn parse_line(line: &str, is_symbol: bool) -> Option<Shape> {
        let first_word = line.split_whitespace().next().unwrap_or("");
        match first_word {
            "LINE" => Line::parse_line(line).map(Shape::Line),
            "RECTANGLE" => Rectangle::parse_line(line).ok().map(Shape::Rectangle),
            "CIRCLE" => Circle::parse_line(line).map(Shape::Circle),
            "ARC" => Arc::parse_line(line).ok().map(Shape::Arc),
            "TEXT" => Text::parse_text_line(line, is_symbol).ok().map(Shape::Text),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_line_parsing() {
        let line_str = "LINE Normal 80 400 80 368 1";
        let line = Line::parse_line(line_str).unwrap();
        assert_eq!(line.start, Point { x: 80, y: 400 });
        assert_eq!(line.end, Point { x: 80, y: 368 });
        assert_eq!(line.style, LineStyle::Dashed);
    }
    #[test]
    fn test_rectangle_parsing() {
        let rect_str = "RECTANGLE Normal 80 448 0 416";
        let rectangle = Rectangle::parse_line(rect_str).unwrap();
        assert_eq!(rectangle.top_left, Point { x: 0, y: 416 });
        assert_eq!(rectangle.bottom_right, Point { x: 80, y: 448 });
        assert_eq!(rectangle.style, LineStyle::Solid);
    }
    #[test]
    fn test_rectangle_with_swapped_points_parsing() {
        let rect_str = "RECTANGLE Normal 336 272 -16 48 2";
        let rectangle = Rectangle::parse_line(rect_str).unwrap();
        assert_eq!(rectangle.top_left, Point { x: -16, y: 48 });
        assert_eq!(rectangle.bottom_right, Point { x: 336, y: 272 });
        assert_eq!(rectangle.style, LineStyle::Dotted);
    }
    #[test]
    fn test_circle_parsing() {
        let circle_str = "CIRCLE Normal 64 528 0 464";
        let circle = Circle::parse_line(circle_str).unwrap();
        assert_eq!(circle.top_left, Point { x: 64, y: 528 });
        assert_eq!(circle.bottom_right, Point { x: 0, y: 464 });
        assert_eq!(circle.style, LineStyle::Solid);
    }
    #[test]
    fn test_arc_parsing() {
        let arc_str = "ARC Normal 0 544 64 608 64 576 0 576";
        let arc = Arc::parse_line(arc_str).unwrap();
        assert_eq!(arc.top_left, Point { x: 0, y: 544 });
        assert_eq!(arc.bottom_right, Point { x: 64, y: 608 });
        assert_eq!(arc.end, Point { x: 64, y: 576 });
        assert_eq!(arc.start, Point { x: 0, y: 576 });
        assert_eq!(arc.style, LineStyle::Solid);
    }
    #[test]
    fn test_shape_parsing() {
        let line_str = "LINE Normal 80 400 80 368 1";
        let shape = Shape::parse_line(line_str, false).unwrap();
        match shape {
            Shape::Line(line) => {
                assert_eq!(line.start, Point { x: 80, y: 400 });
                assert_eq!(line.end, Point { x: 80, y: 368 });
                assert_eq!(line.style, LineStyle::Dashed);
            }
            _ => panic!("Expected Shape::Line variant"),
        }
    }
    #[test]
    fn test_parse_text_with_comment() {
        let line = "TEXT 480 720 Center 2 ;This is a comment";
        let text = Text::parse_text_line(line, false).expect("Failed to parse TEXT line");
        assert_eq!(text.position, Point { x: 480, y: 720 });
        assert_eq!(text.justification, TextJustification::Center);
        assert_eq!(text.size, TextSize::SIZE15);
        assert_eq!(text.text_type, Some(TextType::Comment));
        assert_eq!(text.content, "This is a comment");
    }

    #[test]
    fn test_parse_text_with_spice_directive() {
        let line = "TEXT 100 200 Left 3 !.MODEL NPN N";
        let text = Text::parse_text_line(line, false).expect("Failed to parse TEXT line");
        assert_eq!(text.position, Point { x: 100, y: 200 });
        assert_eq!(text.justification, TextJustification::Left);
        assert_eq!(text.size, TextSize::SIZE20);
        assert_eq!(text.text_type, Some(TextType::SpiceDirective));
        assert_eq!(text.content, ".MODEL NPN N");
    }

    #[test]
    fn test_valid_symbol_text_line() {
        let line = "TEXT -64 0 Center 2 ADI";
        let text = Text::parse_text_line(line, true).expect("Failed to parse TEXT line");
        assert_eq!(text.position, Point { x: -64, y: 0 });
        assert_eq!(text.justification, TextJustification::Center);
        assert_eq!(text.size, TextSize::SIZE15);
        assert_eq!(text.text_type, None); // Default type when no specifier
        assert_eq!(text.content, "ADI");
    }

    #[test]
    fn test_line_break_text() {
        let line = "TEXT -688 -248 Left 2 ;Note: 0.1 μF decoupling capacitors are required \non the primary and secondary supplies. If they \nare driven from the same supply, then one set of\n 0.1 μF decoupling capacitors is sufficient.";
        let text = Text::parse_text_line(line, false).expect("Failed to parse TEXT line");
        assert_eq!(text.position, Point { x: -688, y: -248 });
        assert_eq!(text.justification, TextJustification::Left);
        assert_eq!(text.size, TextSize::SIZE15);
        assert_eq!(text.text_type, Some(TextType::Comment));
        assert_eq!(text.content, "Note: 0.1 μF decoupling capacitors are required \non the primary and secondary supplies. If they \nare driven from the same supply, then one set of\n 0.1 μF decoupling capacitors is sufficient.");
    }

    #[test]
    fn one_character_symbol_text_content() {
        let line = "TEXT -120 128 Left 3 −";
        let text = Text::parse_text_line(line, true).expect("Failed to parse TEXT line");
        assert_eq!(text.position, Point { x: -120, y: 128 });
        assert_eq!(text.justification, TextJustification::Left);
        assert_eq!(text.size, TextSize::SIZE20);
        assert_eq!(text.text_type, None);
        assert_eq!(text.content, "−");
    }
}
