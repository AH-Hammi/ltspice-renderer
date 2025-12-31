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

impl SvgRender for schematic::Schematic {
    fn to_svg(&self) -> Option<Box<dyn svg::node::Node>> {
        let mut group = Group::new().set("id", "schematic");

        for wire in &self.wires {
            if let Some(svg) = wire.to_svg() {
                group = group.add(svg);
            }
        }

        for shape in &self.shapes {
            if let Some(svg) = shape.to_svg() {
                group = group.add(svg);
            }
        }

        for symbol_instance in &self.symbols {
            if let Some(svg) = symbol_instance.to_svg() {
                group = group.add(svg);
            }
        }

        Some(Box::new(group))
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

pub fn generate_svg(schematic: &schematic::Schematic) -> SVG {
    let mut document = Document::new()
        .set("width", schematic.sheet_size.0)
        .set("height", schematic.sheet_size.1)
        .set("font-family", "Arial, sans-serif");

    if let Some(group) = schematic.to_svg() {
        document = document.add(group);
    }
    document
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schematic::Schematic;
    use std::path::PathBuf;
    use std::str::FromStr;
    #[test]
    fn generate_svg_text_sample() {
        let schematic =
            Schematic::from_path(&PathBuf::from_str("test_files/text_sample.asc").unwrap())
                .unwrap();

        let svg_document = generate_svg(&schematic);

        svg::save("text_sample.svg", &svg_document).unwrap();
    }

    #[test]
    fn generate_svg_complex() {
        let schematic =
            Schematic::from_path(&PathBuf::from_str("test_files/complex_sample.asc").unwrap())
                .unwrap();

        let svg_document = generate_svg(&schematic);

        svg::save("complex_sample.svg", &svg_document).unwrap();
    }
}
