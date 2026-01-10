use std::path::PathBuf;

mod bounding_box;
mod file_reader;
mod schematic;
mod shape;
mod svg_renderer;
mod symbol;
mod symbol_loader;

fn main() {
    // load a sample ASC file and parse it
    let document = schematic::Schematic::from_path(&PathBuf::from("test_files/complex_sample.asc"))
        .expect("Failed to read ASC file");
    println!("Parsed document: {:?}", document);
}
