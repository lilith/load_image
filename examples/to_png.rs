use load_image::ImageData;
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from(std::env::args_os().nth(1).expect("File name"));
    let out_path = path.with_extension("example.png");
    let img = load_image::load_image(path, false).unwrap();

    match &img.bitmap {
        ImageData::RGB8(bitmap) => lodepng::encode24_file(&out_path, bitmap, img.width, img.height),
        ImageData::RGBA8(bitmap) => lodepng::encode32_file(&out_path, bitmap, img.width, img.height),
        ImageData::GRAY8(bitmap) => lodepng::encode_file(&out_path, bitmap, img.width, img.height, lodepng::ColorType::GREY, 8),
        ImageData::GRAYA8(bitmap) => lodepng::encode_file(&out_path, bitmap, img.width, img.height, lodepng::ColorType::GREY_ALPHA, 8),
        ImageData::RGB16(_) |
        ImageData::RGBA16(_) |
        ImageData::GRAYA16(_) |
        ImageData::GRAY16(_) => {
            panic!("The input file has 16-bit depth and I was too lazy to implemetn this for the example");
        },
    }.unwrap();
}
