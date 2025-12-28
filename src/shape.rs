#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    start: (i32, i32),
    end: (i32, i32),
    style: LineStyle,
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
        let start_x = parts[2].parse::<i32>().ok()?;
        let start_y = parts[3].parse::<i32>().ok()?;
        let end_x = parts[4].parse::<i32>().ok()?;
        let end_y = parts[5].parse::<i32>().ok()?;
        let style = if parts.len() > 6 {
            LineStyle::from_code(parts[6])
        } else {
            LineStyle::Solid
        };
        Some(Line {
            start: (start_x, start_y),
            end: (end_x, end_y),
            style,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    top_left: (i32, i32),
    bottom_right: (i32, i32),
    style: LineStyle,
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
        let top_left_x = parts[2].parse::<i32>().map_err(|_| "Invalid top left x")?;
        let top_left_y = parts[3].parse::<i32>().map_err(|_| "Invalid top left y")?;
        let bottom_right_x = parts[4]
            .parse::<i32>()
            .map_err(|_| "Invalid bottom right x")?;
        let bottom_right_y = parts[5]
            .parse::<i32>()
            .map_err(|_| "Invalid bottom right y")?;
        let style = if parts.len() > 6 {
            LineStyle::from_code(parts[6])
        } else {
            LineStyle::Solid
        };
        Ok(Rectangle {
            top_left: (top_left_x, top_left_y),
            bottom_right: (bottom_right_x, bottom_right_y),
            style,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Circle {
    top_left: (i32, i32),
    bottom_right: (i32, i32),
    style: LineStyle,
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
        let top_left_x = parts[2].parse::<i32>().ok()?;
        let top_left_y = parts[3].parse::<i32>().ok()?;
        let bottom_right_x = parts[4].parse::<i32>().ok()?;
        let bottom_right_y = parts[5].parse::<i32>().ok()?;
        let style = if parts.len() > 6 {
            LineStyle::from_code(parts[6])
        } else {
            LineStyle::Solid
        };
        Some(Circle {
            top_left: (top_left_x, top_left_y),
            bottom_right: (bottom_right_x, bottom_right_y),
            style,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arc {
    top_left: (i32, i32),
    bottom_right: (i32, i32),
    end: (i32, i32),
    start: (i32, i32),
    style: LineStyle,
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
        let top_left = (
            parts[2].parse::<i32>().map_err(|_| "Invalid top left x")?,
            parts[3].parse::<i32>().map_err(|_| "Invalid top left y")?,
        );
        let bottom_right = (
            parts[4]
                .parse::<i32>()
                .map_err(|_| "Invalid bottom right x")?,
            parts[5]
                .parse::<i32>()
                .map_err(|_| "Invalid bottom right y")?,
        );
        let end = (
            parts[6].parse::<i32>().map_err(|_| "Invalid end x")?,
            parts[7].parse::<i32>().map_err(|_| "Invalid end y")?,
        );
        let start = (
            parts[8].parse::<i32>().map_err(|_| "Invalid start x")?,
            parts[9].parse::<i32>().map_err(|_| "Invalid start y")?,
        );
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
    text_type: Option<TextType>,
    content: String,
}

impl Text {
    fn parse_line(line: &str) -> Option<Text> {
        // The format is TEXT {PositionX} {PositionY} {Justification} {Size} {TypeSpecifier}{Content}
        let parts = line.splitn(6, ' ').collect::<Vec<&str>>();
        if parts.len() < 6 || parts[0] != "TEXT" {
            return None;
        }
        let position_x = parts[1].parse::<i32>().ok()?;
        let position_y = parts[2].parse::<i32>().ok()?;
        let justification = TextJustification::from_str(parts[3])?;
        let size = TextSize::from_str(parts[4])?;

        let content;
        let text_type;
        // Check if the first character of parts[5] is a type specifier
        if let Some(parsed_text_type) = TextType::from_str(&parts[5][..1]) {
            content = &parts[5][1..];
            text_type = Some(parsed_text_type);
        } else {
            text_type = None;
            content = parts[5];
        }
        Some(Text {
            position: (position_x, position_y),
            justification,
            size,
            text_type,
            content: content.to_string(),
        })
    }
}

// present all shapes as a single enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shape {
    Line(Line),
    Rectangle(Rectangle),
    Circle(Circle),
    Arc(Arc),
    Text(Text),
}

impl Shape {
    pub fn parse_line(line: &str) -> Option<Shape> {
        let first_word = line.split_whitespace().next().unwrap_or("");
        match first_word {
            "LINE" => Line::parse_line(line).map(Shape::Line),
            "RECTANGLE" => Rectangle::parse_line(line).ok().map(Shape::Rectangle),
            "CIRCLE" => Circle::parse_line(line).map(Shape::Circle),
            "ARC" => Arc::parse_line(line).ok().map(Shape::Arc),
            "TEXT" => Text::parse_line(line).map(Shape::Text),
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
        assert_eq!(line.start, (80, 400));
        assert_eq!(line.end, (80, 368));
        assert_eq!(line.style, LineStyle::Dashed);
    }
    #[test]
    fn test_rectangle_parsing() {
        let rect_str = "RECTANGLE Normal 80 448 0 416";
        let rectangle = Rectangle::parse_line(rect_str).unwrap();
        assert_eq!(rectangle.top_left, (80, 448));
        assert_eq!(rectangle.bottom_right, (0, 416));
        assert_eq!(rectangle.style, LineStyle::Solid);
    }
    #[test]
    fn test_circle_parsing() {
        let circle_str = "CIRCLE Normal 64 528 0 464";
        let circle = Circle::parse_line(circle_str).unwrap();
        assert_eq!(circle.top_left, (64, 528));
        assert_eq!(circle.bottom_right, (0, 464));
        assert_eq!(circle.style, LineStyle::Solid);
    }
    #[test]
    fn test_arc_parsing() {
        let arc_str = "ARC Normal 0 544 64 608 64 576 0 576";
        let arc = Arc::parse_line(arc_str).unwrap();
        assert_eq!(arc.top_left, (0, 544));
        assert_eq!(arc.bottom_right, (64, 608));
        assert_eq!(arc.end, (64, 576));
        assert_eq!(arc.start, (0, 576));
        assert_eq!(arc.style, LineStyle::Solid);
    }
    #[test]
    fn test_shape_parsing() {
        let line_str = "LINE Normal 80 400 80 368 1";
        let shape = Shape::parse_line(line_str).unwrap();
        match shape {
            Shape::Line(line) => {
                assert_eq!(line.start, (80, 400));
                assert_eq!(line.end, (80, 368));
                assert_eq!(line.style, LineStyle::Dashed);
            }
            _ => panic!("Expected Shape::Line variant"),
        }
    }
    #[test]
    fn test_parse_text_with_comment() {
        let line = "TEXT 480 720 Center 2 ;This is a comment";
        let text = Text::parse_line(line).expect("Failed to parse TEXT line");
        assert_eq!(text.position, (480, 720));
        assert_eq!(text.justification, TextJustification::Center);
        assert_eq!(text.size, TextSize::SIZE15);
        assert_eq!(text.text_type, Some(TextType::Comment));
        assert_eq!(text.content, "This is a comment");
    }

    #[test]
    fn test_parse_text_with_spice_directive() {
        let line = "TEXT 100 200 Left 3 !.MODEL NPN N";
        let text = Text::parse_line(line).expect("Failed to parse TEXT line");
        assert_eq!(text.position, (100, 200));
        assert_eq!(text.justification, TextJustification::Left);
        assert_eq!(text.size, TextSize::SIZE20);
        assert_eq!(text.text_type, Some(TextType::SpiceDirective));
        assert_eq!(text.content, ".MODEL NPN N");
    }

    #[test]
    fn test_valid_symbol_text_line() {
        let line = "TEXT -64 0 Center 2 ADI";
        let text = Text::parse_line(line).expect("Failed to parse TEXT line");
        assert_eq!(text.position, (-64, 0));
        assert_eq!(text.justification, TextJustification::Center);
        assert_eq!(text.size, TextSize::SIZE15);
        assert_eq!(text.text_type, None); // Default type when no specifier
        assert_eq!(text.content, "ADI");
    }
}
