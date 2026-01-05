//! Allows to create SVG renderings of schematics and symbols.
#![allow(dead_code)]

use crate::schematic;
use crate::shape;
use crate::symbol;
use crate::symbol_loader;

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
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, BoundingBox)>;
}

impl SvgRender for shape::Rectangle {
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, BoundingBox)> {
        let rect = Rectangle::new()
            .set("x", self.top_left.0)
            .set("y", self.top_left.1)
            .set("width", self.bottom_right.0 - self.top_left.0)
            .set("height", self.bottom_right.1 - self.top_left.1)
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-dasharray", self.style.to_svg_dasharray());
        let mut bounding_box = BoundingBox::new();
        bounding_box.add_point_i32(self.top_left);
        bounding_box.add_point_i32(self.bottom_right);
        Some((Box::new(rect), bounding_box))
    }
}

impl SvgRender for shape::Circle {
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, BoundingBox)> {
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
        let mut bounding_box = BoundingBox::new();
        bounding_box.add_point_i32(self.top_left);
        bounding_box.add_point_i32(self.bottom_right);
        Some((Box::new(circle), bounding_box))
    }
}

impl SvgRender for shape::Line {
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, BoundingBox)> {
        let line = Line::new()
            .set("x1", self.start.0)
            .set("y1", self.start.1)
            .set("x2", self.end.0)
            .set("y2", self.end.1)
            .set("stroke", "black");

        let mut bounding_box = BoundingBox::new();
        bounding_box.add_point_i32(self.start);
        bounding_box.add_point_i32(self.end);

        Some((Box::new(line), bounding_box))
    }
}

impl SvgRender for shape::Text {
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, BoundingBox)> {
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

        let mut bounding_box = BoundingBox::new();
        bounding_box.add_point_i32(self.position);
        // Calculate the bounding box based on font size and text length
        let text_width = font_size * self.content.len() as f32 * 0.6; // Approximate width
        let text_height = font_size; // Approximate height
                                     // Adjust bounding box based on justification and rotation
        match self.justification {
            shape::TextJustification::Left => {
                bounding_box.add_point((
                    self.position.0 as f32 + text_width,
                    self.position.1 as f32 + text_height,
                ));
                bounding_box.add_point((
                    self.position.0 as f32 + text_width,
                    self.position.1 as f32 - text_height,
                ));
            }
            shape::TextJustification::Center => {
                bounding_box.add_point((
                    self.position.0 as f32 + text_width / 2.,
                    self.position.1 as f32 + text_height,
                ));
                bounding_box.add_point((
                    self.position.0 as f32 - text_width / 2.,
                    self.position.1 as f32,
                ));
            }
            shape::TextJustification::Right => {
                bounding_box.add_point((
                    self.position.0 as f32 - text_width,
                    self.position.1 as f32 + text_height,
                ));
            }
            shape::TextJustification::Top => {
                bounding_box.add_point((
                    self.position.0 as f32 + text_width / 2.,
                    self.position.1 as f32 + text_height,
                ));
                bounding_box.add_point((
                    self.position.0 as f32 - text_width / 2.,
                    self.position.1 as f32,
                ));
            }
            shape::TextJustification::Bottom => {
                bounding_box.add_point((
                    self.position.0 as f32 + text_width / 2.,
                    self.position.1 as f32,
                ));
                bounding_box.add_point((
                    self.position.0 as f32 - text_width / 2.,
                    self.position.1 as f32 - text_height,
                ));
            }
            shape::TextJustification::Invisible => {
                panic!("Invisible text should have been handled earlier")
            }
        }
        Some((Box::new(text), bounding_box))
    }
}

impl SvgRender for Shape {
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, BoundingBox)> {
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
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, BoundingBox)> {
        let line = Line::new()
            .set("x1", self.start.0)
            .set("y1", self.start.1)
            .set("x2", self.end.0)
            .set("y2", self.end.1)
            .set("stroke", "black");
        let mut bounding_box = BoundingBox::new();
        bounding_box.add_point_i32(self.start);
        bounding_box.add_point_i32(self.end);
        Some((Box::new(line), bounding_box))
    }
}

impl SvgRender for symbol_loader::Pin {
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, BoundingBox)> {
        // Call the implementation of the inner text element
        todo!()
    }
}

impl SvgRender for symbol_loader::LibrarySymbol {
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, BoundingBox)> {
        let mut symbol = Symbol::new().set("id", "library_symbol");

        let mut bounding_box = BoundingBox::new();

        for shape in &self.shapes {
            if let Some((svg, shape_bounding_box)) = shape.to_svg() {
                symbol = symbol.add(svg);
                bounding_box.merge(&shape_bounding_box);
            }
        }
        for pin in &self.pins {
            if let Some((svg, pin_bounding_box)) = pin.to_svg() {
                symbol = symbol.add(svg);
                bounding_box.merge(&pin_bounding_box);
            }
        }

        Some((Box::new(symbol), bounding_box))
    }
}

impl SvgRender for symbol::Symbol {
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, BoundingBox)> {
        let mut group = Group::new().set("id", "symbol");

        group = group.add(
            svg::node::element::Use::new()
                .set("href", self.symbol_id.as_str())
                .set("x", self.position.0)
                .set("y", self.position.1)
                .set("transform", format!("rotate({})", self.rotation)),
        );
        let mut bounding_box = BoundingBox::new();
        bounding_box.add_point_i32(self.position);

        Some((Box::new(group), bounding_box))
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

    fn add_point_i32(&mut self, point: (i32, i32)) {
        let point_f32 = (point.0 as f32, point.1 as f32);
        self.add_point(point_f32);
    }

    fn merge(&mut self, other: &BoundingBox) {
        self.add_point(other.top_left);
        self.add_point(other.bottom_right);
    }
}

pub fn generate_svg(schematic: &schematic::Schematic) -> SVG {
    let mut document = Document::new().set("font-family", "Arial, sans-serif");

    let mut schematic_bounding_box = BoundingBox::new();

    for wire in &schematic.wires {
        if let Some((svg, local_bounding_box)) = wire.to_svg() {
            document = document.add(svg);
            schematic_bounding_box.merge(&local_bounding_box);
        }
    }

    for shape in &schematic.shapes {
        if let Some((svg, local_bounding_box)) = shape.to_svg() {
            document = document.add(svg);
            schematic_bounding_box.merge(&local_bounding_box);
        }
    }

    for symbol_instance in &schematic.symbols {
        if let Some((svg, local_bounding_box)) = symbol_instance.to_svg() {
            document = document.add(svg);
            schematic_bounding_box.merge(&local_bounding_box);
        }
    }

    println!(
        "Schematic bounding box: top_left=({:.2}, {:.2}), bottom_right=({:.2}, {:.2})",
        schematic_bounding_box.top_left.0,
        schematic_bounding_box.top_left.1,
        schematic_bounding_box.bottom_right.0,
        schematic_bounding_box.bottom_right.1
    );

    document
        .set(
            "width",
            schematic_bounding_box.bottom_right.0 - schematic_bounding_box.top_left.0,
        )
        .set(
            "height",
            schematic_bounding_box.bottom_right.1 - schematic_bounding_box.top_left.1,
        )
        .set(
            "viewBox",
            format!(
                "{} {} {} {}",
                schematic_bounding_box.top_left.0,
                schematic_bounding_box.top_left.1,
                schematic_bounding_box.bottom_right.0 - schematic_bounding_box.top_left.0,
                schematic_bounding_box.bottom_right.1 - schematic_bounding_box.top_left.1
            ),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schematic::Schematic;
    use std::path::PathBuf;
    use std::str::FromStr;

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
    fn shape_sample() {
        let schematic =
            Schematic::from_path(&PathBuf::from_str("test_files/shape_sample.asc").unwrap())
                .unwrap();

        let svg_document = generate_svg(&schematic);

        svg::save("shape_sample.svg", &svg_document).unwrap();
    }

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
}
