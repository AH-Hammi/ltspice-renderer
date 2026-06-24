use std::path::Path;

use crate::svg_renderer::generate_svg_from_schematic;

mod bounding_box;
mod file_reader;
mod schematic;
mod shape;
mod svg_renderer;
mod symbol;
mod symbol_loader;

fn main() {
    let complex_sample = Path::new("test_files/complex_sample.asc");
    // load a sample ASC file and parse it
    let document =
        schematic::Schematic::from_path(complex_sample).expect("Failed to read ASC file");
    println!("Parsed document: {:?}", document);
    let svg_document = generate_svg_from_schematic(complex_sample);

    svg::save("complex_sample.svg", &svg_document.unwrap()).unwrap();
}
