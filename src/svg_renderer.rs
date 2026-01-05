//! Allows to create SVG renderings of schematics and symbols.
#![allow(dead_code)]

use crate::schematic;
use crate::shape;
use crate::symbol;

use svg::Document;

use svg::node::element::Circle;
use svg::node::element::Group;
use svg::node::element::Line;
use svg::node::element::Rectangle;
use svg::node::element::Symbol;
use svg::node::element::Text;
use svg::node::element::SVG;
use svg::Node;

use crate::shape::Shape;

trait SvgRender {
    fn to_svg(&self) -> Option<Box<dyn svg::node::Node>>;
}

impl SvgRender for shape::Rectangle {
    fn to_svg(&self) -> Option<Box<dyn svg::node::Node>> {
        let rect = Rectangle::new()
            .set("x", self.top_left.0)
            .set("y", self.top_left.1)
            .set("width", self.bottom_right.0 - self.top_left.0)
            .set("height", self.bottom_right.1 - self.top_left.1)
            .set("fill", "none")
            .set("stroke", "black");

        Some(Box::new(rect))
    }
}

impl SvgRender for shape::Circle {
    fn to_svg(&self) -> Option<Box<dyn svg::node::Node>> {
        let circle = Circle::new()
            .set(
                "cx",
                self.top_left.0 + (self.bottom_right.0 - self.top_left.0) / 2,
            )
            .set(
                "cy",
                self.top_left.1 + (self.bottom_right.1 - self.top_left.1) / 2,
            )
            .set("r", (self.bottom_right.0 - self.top_left.0) / 2)
            .set("fill", "none")
            .set("stroke", "black");

        Some(Box::new(circle))
    }
}

impl SvgRender for shape::Line {
    fn to_svg(&self) -> Option<Box<dyn svg::node::Node>> {
        let line = Line::new()
            .set("x1", self.start.0)
            .set("y1", self.start.1)
            .set("x2", self.end.0)
            .set("y2", self.end.1)
            .set("stroke", "black");

        Some(Box::new(line))
    }
}

impl SvgRender for shape::Text {
    fn to_svg(&self) -> Option<Box<dyn svg::node::Node>> {
        if self.justification == shape::TextJustification::Invisible {
            return None;
        }

        let font_size = self.size.multiplier() * 14.0;

        let y_offset = match self.justification {
            shape::TextJustification::Top => font_size,
            shape::TextJustification::Center
            | shape::TextJustification::Left
            | shape::TextJustification::Right => font_size / 2.,
            shape::TextJustification::Bottom => 0.,
            shape::TextJustification::Invisible => {
                panic!("Invisible text should have been handled earlier")
            }
        };
        let mut text = Text::new(self.content.as_str())
            .set("x", self.position.0)
            .set("y", self.position.1 as f32 + y_offset)
            .set("font-size", format!("{}px", font_size))
            .set(
                "text-anchor",
                match self.justification {
                    shape::TextJustification::Left => "start",
                    shape::TextJustification::Center
                    | shape::TextJustification::Bottom
                    | shape::TextJustification::Top => "middle",
                    shape::TextJustification::Right => "end",
                    shape::TextJustification::Invisible => {
                        panic!("Invisible text should have been handled earlier")
                    }
                },
            );

        // Rotate 90 degrees for vertical text
        if self.vertical {
            text = text.set(
                "transform",
                format!("rotate(-90, {}, {})", self.position.0, self.position.1),
            );
        }
        Some(Box::new(text))
    }
}

impl SvgRender for Shape {
    fn to_svg(&self) -> Option<Box<dyn svg::node::Node>> {
        match self {
            Shape::Rectangle(shape) => shape.to_svg(),
            Shape::Circle(shape) => shape.to_svg(),
            Shape::Line(shape) => shape.to_svg(),
            Shape::Text(shape) => shape.to_svg(),
            _ => None,
        }
    }
}

impl SvgRender for schematic::Wire {
    fn to_svg(&self) -> Option<Box<dyn svg::node::Node>> {
        let line = Line::new()
            .set("x1", self.start.0)
            .set("y1", self.start.1)
            .set("x2", self.end.0)
            .set("y2", self.end.1)
            .set("stroke", "black");
        Some(Box::new(line))
    }
}

impl SvgRender for symbol::LibrarySymbol {
    fn to_svg(&self) -> Option<Box<dyn svg::node::Node>> {
        let mut symbol = Symbol::new().set("id", "library_symbol");

        for shape in &self.shapes {
            if let Some(svg) = shape.to_svg() {
                symbol = symbol.add(svg);
            }
        }

        Some(Box::new(symbol))
    }
}

impl SvgRender for symbol::Symbol {
    fn to_svg(&self) -> Option<Box<dyn svg::node::Node>> {
        let mut group = Group::new().set("id", "symbol");

        group = group.add(
            svg::node::element::Use::new()
                .set("href", self.symbol_id.as_str())
                .set(
                    "transform",
                    format!(
                        "translate({}, {}) rotate({})",
                        self.position.0, self.position.1, self.rotation
                    ),
                ),
        );

        Some(Box::new(group))
    }
}

struct BoundingBox {
    top_left: (f32, f32),
    bottom_right: (f32, f32),
}

impl BoundingBox {
    fn new() -> Self {
        BoundingBox {
            top_left: (f32::MAX, f32::MAX),
            bottom_right: (f32::MIN, f32::MIN),
        }
    }
    fn add_point(&mut self, point: (f32, f32)) {
        if point.0 < self.top_left.0 {
            self.top_left.0 = point.0;
        }
        if point.1 < self.top_left.1 {
            self.top_left.1 = point.1;
        }
        if point.0 > self.bottom_right.0 {
            self.bottom_right.0 = point.0;
        }
        if point.1 > self.bottom_right.1 {
            self.bottom_right.1 = point.1;
        }
    }

    fn merge(&mut self, other: &BoundingBox) {
        self.add_point(other.top_left);
        self.add_point(other.bottom_right);
    }
}

macro_rules! get_attr {
    ($element:expr, $attr:expr) => {
        $element.get_attributes().unwrap().get($attr).unwrap()
    };
}
// Makro to retrieve attribute values as f32
macro_rules! get_attr_f32 {
    ($element:expr, $attr:expr) => {
        get_attr!($element, $attr).parse::<f32>().unwrap()
    };
}

fn get_bounding_box_node(node: &dyn Node) -> Option<BoundingBox> {
    let mut node_bounding_box = BoundingBox::new();
    for child in node.get_children().unwrap() {
        let child = &**child;
        if let Some(bounding_box) = match child.get_name() {
            "line" => {
                let x1: f32 = get_attr_f32!(child, "x1");
                let y1: f32 = get_attr_f32!(child, "y1");
                let x2: f32 = get_attr_f32!(child, "x2");
                let y2: f32 = get_attr_f32!(child, "y2");

                let mut bounding_box = BoundingBox::new();
                bounding_box.add_point((x1, y1));
                bounding_box.add_point((x2, y2));
                Some(bounding_box)
            }
            "rect" => {
                let x: f32 = get_attr_f32!(child, "x");
                let y: f32 = get_attr_f32!(child, "y");
                let width: f32 = get_attr_f32!(child, "width");
                let height: f32 = get_attr_f32!(child, "height");

                let mut bounding_box = BoundingBox::new();
                bounding_box.add_point((x, y));
                bounding_box.add_point((x + width, y + height));
                Some(bounding_box)
            }
            "text" => {
                let x: f32 = get_attr_f32!(child, "x");
                let y: f32 = get_attr_f32!(child, "y");
                let _font_size: f32 = get_attr_f32!(child, "font-size");
                let _text_anchor = get_attr!(child, "text-anchor").to_string();
                // For simplicity, we treat text as a point at (x, y)
                let mut bounding_box = BoundingBox::new();
                bounding_box.add_point((x, y));
                Some(bounding_box)
            }
            _ => {
                println!(
                    "Warning: Bounding box calculation for element '{}' not implemented.",
                    child.get_name()
                );
                None
            }
        } {
            node_bounding_box.merge(&bounding_box);
        }
    }
    Some(node_bounding_box)
}

pub fn generate_svg(schematic: &schematic::Schematic) -> SVG {
    let mut document = Document::new()
        .set("width", schematic.sheet_size.0)
        .set("height", schematic.sheet_size.1)
        .set("font-family", "Arial, sans-serif");

    for wire in &schematic.wires {
        if let Some(svg) = wire.to_svg() {
            document = document.add(svg);
        }
    }

    for shape in &schematic.shapes {
        if let Some(svg) = shape.to_svg() {
            document = document.add(svg);
        }
    }

    for symbol_instance in &schematic.symbols {
        if let Some(svg) = symbol_instance.to_svg() {
            document = document.add(svg);
        }
    }

    let bounding_box = get_bounding_box_node(&document).unwrap();

    println!(
        "Schematic bounding box: top_left=({:.2}, {:.2}), bottom_right=({:.2}, {:.2})",
        bounding_box.top_left.0,
        bounding_box.top_left.1,
        bounding_box.bottom_right.0,
        bounding_box.bottom_right.1
    );

    document
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schematic::Schematic;
    use std::path::PathBuf;
    use std::str::FromStr;
    #[test]
    fn text_sample() {
        let schematic =
            Schematic::from_path(&PathBuf::from_str("test_files/text_sample.asc").unwrap())
                .unwrap();

        let svg_document = generate_svg(&schematic);

        svg::save("text_sample.svg", &svg_document).unwrap();
    }

    #[test]
    fn complex_sample() {
        let schematic =
            Schematic::from_path(&PathBuf::from_str("test_files/complex_sample.asc").unwrap())
                .unwrap();

        let svg_document = generate_svg(&schematic);

        svg::save("complex_sample.svg", &svg_document).unwrap();
    }

    #[test]
    fn bounding_box_merge() {
        let mut bb1 = BoundingBox::new();
        bb1.add_point((10.0, 10.0));
        bb1.add_point((20.0, 20.0));

        let mut bb2 = BoundingBox::new();
        bb2.add_point((15.0, 5.0));
        bb2.add_point((25.0, 15.0));

        bb1.merge(&bb2);

        assert_eq!(bb1.top_left, (10.0, 5.0));
        assert_eq!(bb1.bottom_right, (25.0, 20.0));
    }

    #[test]
    fn document_bounding_box_calculation() {
        let mut document = Document::new();
        let line = Line::new()
            .set("x1", 10)
            .set("y1", 20)
            .set("x2", 30)
            .set("y2", 40)
            .set("stroke", "black");
        let rectangle = Rectangle::new()
            .set("x", 15)
            .set("y", 25)
            .set("width", 50)
            .set("height", 30)
            .set("fill", "none")
            .set("stroke", "black");
        document = document.add(line).add(rectangle);
        let bounding_box = get_bounding_box_node(&document).unwrap();
        assert_eq!(bounding_box.top_left, (10.0, 20.0));
        assert_eq!(bounding_box.bottom_right, (65.0, 55.0));
    }

    #[test]
    fn bounding_box_text_sample() {
        let schematic =
            Schematic::from_path(&PathBuf::from_str("test_files/text_sample.asc").unwrap())
                .unwrap();

        let svg_document = generate_svg(&schematic);

        let bounding_box = get_bounding_box_node(&svg_document).unwrap();

        assert_eq!(bounding_box.top_left, (0.0, 0.0));
        assert_eq!(bounding_box.bottom_right, (800.0, 600.0));
    }
}
