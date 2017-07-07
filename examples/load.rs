extern crate load_image;

fn main() {
    let path = std::env::args().nth(1).expect("File name");
    load_image::load_image(path, false).unwrap();
}
