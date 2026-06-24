//! Allows to create SVG renderings of schematics and symbols.
#![allow(dead_code)]

use std::collections::HashMap;
use std::io::Error;
use std::path::Path;

use crate::bounding_box::BoundingBox;
use crate::schematic;
use crate::schematic::Schematic;
use crate::shape;
use crate::shape::Point;
use crate::symbol;
use crate::symbol_loader;

use svg::Document;

use svg::node::element::Circle;
use svg::node::element::Group;
use svg::node::element::Line;
use svg::node::element::Rectangle;
use svg::node::element::Symbol;
use svg::node::element::Text;
use svg::node::element::Use;
use svg::node::element::SVG;

use crate::shape::Shape;

trait SvgRender {
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, BoundingBox)>;
}

impl SvgRender for shape::Rectangle {
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, BoundingBox)> {
        let stroke_width = 1.;
        let rect = Rectangle::new()
            .set("x", self.top_left.x)
            .set("y", self.top_left.y)
            .set("width", self.bottom_right.x - self.top_left.x)
            .set("height", self.bottom_right.y - self.top_left.y)
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-linecap", "round")
            .set("stroke-width", stroke_width)
            .set("stroke-dasharray", self.style.to_svg_dasharray());
        let mut bounding_box = BoundingBox::new();
        bounding_box.add_point_i32(self.top_left);
        bounding_box.add_point_i32(self.bottom_right);
        bounding_box.expand(stroke_width / 2.);
        Some((Box::new(rect), bounding_box))
    }
}

impl SvgRender for shape::Circle {
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, BoundingBox)> {
        let circle = Circle::new()
            .set(
                "cx",
                self.top_left.x + (self.bottom_right.x - self.top_left.x) / 2,
            )
            .set(
                "cy",
                self.top_left.y + (self.bottom_right.y - self.top_left.y) / 2,
            )
            .set("r", (self.bottom_right.x - self.top_left.x) / 2)
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
            .set("x1", self.start.x)
            .set("y1", self.start.y)
            .set("x2", self.end.x)
            .set("y2", self.end.y)
            .set("stroke", "black")
            .set("stroke-width", "1")
            .set("stroke-linecap", "round")
            .set("stroke-dasharray", self.style.to_svg_dasharray());

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

        let x_offset = match self.justification {
            shape::TextJustification::Left => self.offset,
            shape::TextJustification::Center
            | shape::TextJustification::Top
            | shape::TextJustification::Bottom => self.offset,
            shape::TextJustification::Right => -self.offset,
            shape::TextJustification::Invisible => {
                panic!("Invisible text should have been handled earlier")
            }
        };

        let mut text = Text::new(self.content.as_str())
            .set("x", self.position.x)
            .set("y", self.position.y)
            .set("dx", x_offset)
            .set("dy", y_offset)
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
                format!("rotate(-90, {}, {})", self.position.x, self.position.y),
            );
        }

        let mut bounding_box = BoundingBox::new();
        bounding_box.add_point_i32(self.position);
        // Calculate the bounding box based on font size and text length
        let text_width = font_size * self.content.len() as f32 * 0.5 + self.offset as f32; // Approximate width
        let text_height = font_size; // Approximate height

        // Adjust bounding box based on justification and rotation
        match self.justification {
            shape::TextJustification::Left => {
                if self.vertical {
                    // Extend the bounding box to the right top edge of the text
                    bounding_box.add_point((
                        self.position.x as f32 - text_height / 2.,
                        self.position.y as f32 - text_width,
                    ));
                    // Extend the bounding box to the bottom right edge of the text
                    bounding_box.add_point((
                        self.position.x as f32 + text_height / 2.,
                        self.position.y as f32,
                    ));
                } else {
                    // Top right of the text
                    bounding_box.add_point((
                        self.position.x as f32 + text_width,
                        self.position.y as f32 - text_height / 2.,
                    ));
                    // Bottom left of the text
                    bounding_box.add_point((
                        self.position.x as f32,
                        self.position.y as f32 + text_height / 2.,
                    ));
                }
            }
            shape::TextJustification::Center => {
                if self.vertical {
                    // Top left of the text
                    bounding_box.add_point((
                        self.position.x as f32 - text_height / 2.,
                        self.position.y as f32 - text_width / 2.,
                    ));
                    // Bottom right of the text
                    bounding_box.add_point((
                        self.position.x as f32 + text_height / 2.,
                        self.position.y as f32 + text_width / 2.,
                    ));
                } else {
                    // Top Left
                    bounding_box.add_point((
                        self.position.x as f32 - text_width / 2.,
                        self.position.y as f32 - text_height / 2.,
                    ));
                    // Bottom Right
                    bounding_box.add_point((
                        self.position.x as f32 + text_width / 2.,
                        self.position.y as f32 + text_height / 2.,
                    ));
                }
            }
            shape::TextJustification::Right => {
                if self.vertical {
                    // Top Left
                    bounding_box.add_point((
                        self.position.x as f32 - text_height / 2.,
                        self.position.y as f32,
                    ));
                    // Bottom Right
                    bounding_box.add_point((
                        self.position.x as f32 + text_height / 2.,
                        self.position.y as f32 + text_width,
                    ));
                } else {
                    // Top Left
                    bounding_box.add_point((
                        self.position.x as f32 - text_width,
                        self.position.y as f32 - text_height / 2.,
                    ));
                    // Bottom Right
                    bounding_box.add_point((
                        self.position.x as f32,
                        self.position.y as f32 + text_height / 2.,
                    ));
                }
            }
            shape::TextJustification::Top => {
                if self.vertical {
                    // Top Left
                    bounding_box.add_point((
                        self.position.x as f32,
                        self.position.y as f32 - text_width / 2.,
                    ));
                    // Bottom Right
                    bounding_box.add_point((
                        self.position.x as f32 + text_height,
                        self.position.y as f32 + text_width / 2.,
                    ));
                } else {
                    // Top Left
                    bounding_box.add_point((
                        self.position.x as f32 - text_width / 2.,
                        self.position.y as f32,
                    ));
                    // Bottom Right
                    bounding_box.add_point((
                        self.position.x as f32 + text_width / 2.,
                        self.position.y as f32 + text_height,
                    ));
                }
            }
            shape::TextJustification::Bottom => {
                if self.vertical {
                    bounding_box.add_point((
                        self.position.x as f32 - text_height,
                        self.position.y as f32 - text_width / 2.,
                    ));
                    bounding_box.add_point((
                        self.position.x as f32,
                        self.position.y as f32 + text_width / 2.,
                    ));
                } else {
                    bounding_box.add_point((
                        self.position.x as f32 - text_width / 2.,
                        self.position.y as f32 - text_height,
                    ));
                    bounding_box.add_point((
                        self.position.x as f32 + text_width / 2.,
                        self.position.y as f32,
                    ));
                }
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
            .set("x1", self.start.x)
            .set("y1", self.start.y)
            .set("x2", self.end.x)
            .set("y2", self.end.y)
            .set("stroke", "black")
            .set("stroke-width", 1)
            .set("stroke-linecap", "round");
        let mut bounding_box = BoundingBox::new();
        bounding_box.add_point_i32(self.start);
        bounding_box.add_point_i32(self.end);
        Some((Box::new(line), bounding_box))
    }
}

impl SvgRender for symbol_loader::Pin {
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, BoundingBox)> {
        // Call the implementation of the inner text element
        self.text.to_svg()
    }
}

impl symbol_loader::LibrarySymbol {
    fn to_svg(&self, id: &str) -> Option<(Box<dyn svg::node::Node>, BoundingBox)> {
        let mut symbol = Symbol::new().set("id", id);

        let mut bounding_box = BoundingBox::new();
        // Add the origin to the bounding box
        bounding_box.add_point_i32(Point::default());

        // Add circle on the origin
        let origin_circle = Circle::new()
            .set("cx", 0)
            .set("cy", 0)
            .set("r", 1)
            .set("fill", "none")
            .set("stroke", "black");
        symbol = symbol.add(origin_circle);

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
        symbol = symbol
            .set(
                "width",
                bounding_box.bottom_right.0 - bounding_box.top_left.0,
            )
            .set(
                "height",
                bounding_box.bottom_right.1 - bounding_box.top_left.1,
            )
            .set(
                "viewBox",
                format!(
                    "{} {} {} {}",
                    bounding_box.top_left.0,
                    bounding_box.top_left.1,
                    bounding_box.bottom_right.0 - bounding_box.top_left.0,
                    bounding_box.bottom_right.1 - bounding_box.top_left.1
                ),
            );

        Some((Box::new(symbol), bounding_box))
    }
}

impl symbol::Symbol {
    fn to_svg(
        &self,
        library_bounding_box: BoundingBox,
    ) -> Option<(Box<dyn svg::node::Node>, BoundingBox)> {
        let mut bounding_box = library_bounding_box;
        let scale = if self.rotation.is_mirror() { -1.0 } else { 1.0 };
        let use_symbol = Use::new()
            .set("href", format!("#{}", self.symbol_id.as_str()))
            .set(
                "transform",
                format!(
                    "translate({x},{y}) rotate({rotation}) scale(1,{scale}) ",
                    x = self.position.0,
                    y = self.position.1,
                    rotation = self.rotation.as_int(),
                    scale = scale,
                ),
            );
        // Add windowing attributes

        bounding_box.translate((self.position.0 as f32, self.position.1 as f32));

        Some((Box::new(use_symbol), bounding_box))
    }
}

impl symbol_loader::SymbolLoader {
    fn to_svg(&self) -> Option<(Box<dyn svg::node::Node>, HashMap<String, BoundingBox>)> {
        let mut group = Group::new();
        let mut bounding_box_per_id: HashMap<String, BoundingBox> = HashMap::new();
        for (id, library_symbol) in self.used_symbols.iter() {
            if let Some((svg, symbol_bounding_box)) = library_symbol.to_svg(id) {
                group = group.add(svg);
                bounding_box_per_id.insert(id.clone(), symbol_bounding_box);
            }
        }
        Some((Box::new(group), bounding_box_per_id))
    }
}

struct ConnectedWire {
    wires: Vec<schematic::Wire>,
}

impl ConnectedWire {
    fn new() -> Self {
        Self { wires: Vec::new() }
    }

    fn is_connected(&self, point: Point) -> bool {
        self.wires
            .iter()
            .any(|wire| wire.start == point || wire.end == point)
    }

    fn add(&mut self, wire: schematic::Wire) {
        self.wires.push(wire);
    }

    fn combine(&mut self, other: Self) {
        self.wires.extend(other.wires);
    }

    fn on_wire(&self, flag: schematic::Flag) -> bool {
        // Check if the flag is on a wire.
        // Each wire is a line between two points.
        for wire in self.wires.iter() {
            // Check if x is between the start and end point
            return (wire.start.x < flag.position.0 && flag.position.0 < wire.end.x)
                || (wire.end.x < flag.position.0 && flag.position.0 < wire.start.x);
        }
        return false;
    }
}

pub fn generate_svg_from_schematic(schematic_path: &Path) -> Result<SVG, Error> {
    let mut schematic = Schematic::from_path(schematic_path).unwrap();
    let mut document = Document::new()
        .set("font-family", "Arial, sans-serif")
        .set("font-weight", "bold");

    let mut schematic_bounding_box = BoundingBox::new();

    let mut connected_wires: Vec<ConnectedWire> = Vec::new();

    // Build the connected_wires map
    for wire in &schematic.wires {
        let start = wire.start;
        let end = wire.end;
        for connected_wire in connected_wires.iter_mut() {
            if connected_wire.is_connected(start) || connected_wire.is_connected(end) {
                connected_wire.add(wire.clone());
                break;
            }
        }
        if !connected_wires
            .iter()
            .any(|wire| wire.is_connected(start) || wire.is_connected(end))
        {
            connected_wires.push(ConnectedWire::new());
            connected_wires.last_mut().unwrap().add(wire.clone());
        }
    }

    for shape in &schematic.shapes {
        if let Some((svg, local_bounding_box)) = shape.to_svg() {
            document = document.add(svg);
            schematic_bounding_box.merge(&local_bounding_box);
        }
    }

    // Add all symbols to used
    let mut symbol_loader = symbol_loader::SymbolLoader::new(Some(vec![schematic_path
        .parent()
        .unwrap()
        .to_path_buf()]));
    schematic.make_symbols_absolute(&mut symbol_loader)?;

    let mut bounding_boxes_for_symbols: Option<HashMap<String, BoundingBox>> = None;
    // Render all used symbols
    if let Some((svg, local_bounding_box_per_id)) = symbol_loader.to_svg() {
        document = document.add(svg);
        bounding_boxes_for_symbols = Some(local_bounding_box_per_id);
    }

    let bounding_boxes_for_symbols =
        bounding_boxes_for_symbols.expect("Bounding boxes for symbols should be available");

    for symbol_instance in &schematic.symbols {
        let bounding_box_for_symbol = bounding_boxes_for_symbols
            .get(&symbol_instance.symbol_id)
            .cloned()
            .expect("Bounding box for symbol instance should be available");
        if let Some((svg, bounding_box)) = symbol_instance.to_svg(bounding_box_for_symbol) {
            document = document.add(svg);
            schematic_bounding_box.merge(&bounding_box);
        }
    }

    println!(
        "Schematic bounding box: top_left=({:.2}, {:.2}), bottom_right=({:.2}, {:.2})",
        schematic_bounding_box.top_left.0,
        schematic_bounding_box.top_left.1,
        schematic_bounding_box.bottom_right.0,
        schematic_bounding_box.bottom_right.1
    );

    Ok(document
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
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let path = Path::new("test_files/shape_sample.asc");

        let svg_document = generate_svg_from_schematic(&path);

        svg::save("shape_sample.svg", &svg_document.unwrap()).unwrap();
    }

    #[test]
    fn text_sample() {
        let path = Path::new("test_files/text_sample.asc");

        let svg_document = generate_svg_from_schematic(&path);

        svg::save("text_sample.svg", &svg_document.unwrap()).unwrap();
    }

    #[test]
    fn complex_sample() {
        let path = Path::new("test_files/complex_sample.asc");

        let svg_document = generate_svg_from_schematic(&path);

        svg::save("complex_sample.svg", &svg_document.unwrap()).unwrap();
    }
}
