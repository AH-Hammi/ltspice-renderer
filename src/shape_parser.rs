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
    pub fn from_asc_line(line: &str) -> Option<Line> {
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
    pub fn from_asc_line(line: &str) -> Result<Rectangle, &'static str> {
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
    pub fn from_asc_line(line: &str) -> Option<Circle> {
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
    pub fn from_asc_line(line: &str) -> Result<Arc, &'static str> {
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

// present all shapes as a single enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Line(Line),
    Rectangle(Rectangle),
    Circle(Circle),
    Arc(Arc),
}

impl Shape {
    pub fn from_asc_line(line: &str) -> Option<Shape> {
        if line.starts_with("LINE") {
            Line::from_asc_line(line).map(Shape::Line)
        } else if line.starts_with("RECTANGLE") {
            Rectangle::from_asc_line(line).ok().map(Shape::Rectangle)
        } else if line.starts_with("CIRCLE") {
            Circle::from_asc_line(line).map(Shape::Circle)
        } else if line.starts_with("ARC") {
            Arc::from_asc_line(line).ok().map(Shape::Arc)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_line_parsing() {
        let line_str = "LINE Normal 80 400 80 368 1";
        let line = Line::from_asc_line(line_str).unwrap();
        assert_eq!(line.start, (80, 400));
        assert_eq!(line.end, (80, 368));
        assert_eq!(line.style, LineStyle::Dashed);
    }
    #[test]
    fn test_rectangle_parsing() {
        let rect_str = "RECTANGLE Normal 80 448 0 416";
        let rectangle = Rectangle::from_asc_line(rect_str).unwrap();
        assert_eq!(rectangle.top_left, (80, 448));
        assert_eq!(rectangle.bottom_right, (0, 416));
        assert_eq!(rectangle.style, LineStyle::Solid);
    }
    #[test]
    fn test_circle_parsing() {
        let circle_str = "CIRCLE Normal 64 528 0 464";
        let circle = Circle::from_asc_line(circle_str).unwrap();
        assert_eq!(circle.top_left, (64, 528));
        assert_eq!(circle.bottom_right, (0, 464));
        assert_eq!(circle.style, LineStyle::Solid);
    }
    #[test]
    fn test_arc_parsing() {
        let arc_str = "ARC Normal 0 544 64 608 64 576 0 576";
        let arc = Arc::from_asc_line(arc_str).unwrap();
        assert_eq!(arc.top_left, (0, 544));
        assert_eq!(arc.bottom_right, (64, 608));
        assert_eq!(arc.end, (64, 576));
        assert_eq!(arc.start, (0, 576));
        assert_eq!(arc.style, LineStyle::Solid);
    }
    #[test]
    fn test_shape_parsing() {
        let line_str = "LINE Normal 80 400 80 368 1";
        let shape = Shape::from_asc_line(line_str).unwrap();
        match shape {
            Shape::Line(line) => {
                assert_eq!(line.start, (80, 400));
                assert_eq!(line.end, (80, 368));
                assert_eq!(line.style, LineStyle::Dashed);
            }
            _ => panic!("Expected Shape::Line variant"),
        }
    }
}
