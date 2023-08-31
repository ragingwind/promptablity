use extractor::extractor;
use std::{env, fs};

macro_rules! current_sample {($fname:expr) => (
  concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/samples/", $fname) // assumes Linux ('/')!
)}

#[test]
fn it_tests_with_simple() {
    let url = url::Url::parse("https://www.example.com").unwrap();
    let sample = current_sample!("simple.html");
    let html = fs::read_to_string(sample).unwrap();

    match extractor::extract(&mut html.as_bytes(), &url) {
        Ok(true) => assert!(true),
        _ => assert!(false)
    }
}