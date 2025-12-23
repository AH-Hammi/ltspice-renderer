#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

mod asc_parser;
mod asy_parser;
mod shape_parser;

fn main() {
    // load a sample ASC file and parse it
    let asc_content =
        std::fs::read_to_string("test_files/complex_sample.asc").expect("Failed to read ASC file");
    let document = asc_parser::AcsDocument::parse(&asc_content);
    println!("Parsed document: {:?}", document);
}
