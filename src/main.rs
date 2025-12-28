#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

mod asc_document;
mod asy_document;
mod file_reader;
mod shape;
mod symbol;

fn main() {
    // load a sample ASC file and parse it
    let document = asc_document::AscDocument::from_path("test_files/complex_sample.asc")
        .expect("Failed to read ASC file");
    println!("Parsed document: {:?}", document);
}
