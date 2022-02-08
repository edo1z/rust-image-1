use image::GenericImageView;
use image::ImageFormat;
use std::fs::File;
use std::path::Path;

fn main() {
    let file_path = "src/img.png";
    let ext = Path::new(file_path).extension().unwrap().to_str().unwrap();
    let img_format = match ext {
        "png" => ImageFormat::Png,
        _ => ImageFormat::Jpeg,
    };
    let bytes: Vec<u8> = std::fs::read(file_path).unwrap();
    match image::load_from_memory_with_format(&bytes, img_format) {
        Ok(img) => {
            println!("dimensions {:?}", img.dimensions());
            let bytes = img.as_bytes();
            println!("{:?}", &bytes[0..100]);

            let new_img = img.thumbnail(300, 300).blur(2.0);
            let mut output = File::create(format!("new_img.{}", ext)).unwrap();
            new_img.write_to(&mut output, img_format).unwrap();
        }
        Err(_) => {
            println!("Invalid File");
        }
    }
}
