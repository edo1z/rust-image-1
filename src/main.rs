use image::ImageFormat;
use std::fs::File;
use std::path::Path;

fn main() {
    let file_path = "src/img.jpg";
    let ext = Path::new(file_path).extension().unwrap().to_str().unwrap();
    let img_format = match ext {
        "png" => ImageFormat::Png,
        _ => ImageFormat::Jpeg,
    };
    let bytes: Vec<u8> = std::fs::read(file_path).unwrap();
    let orientation = get_orientation(&bytes).unwrap();
    println!("Orientation: {orientation}");
    match image::load_from_memory_with_format(&bytes, img_format) {
        Ok(img) => {
            let new_img = img.thumbnail(300, 300).blur(2.0);
            let mut output = File::create(format!("new_img.{}", ext)).unwrap();
            new_img.write_to(&mut output, img_format).unwrap();
        }
        Err(_) => {
            println!("Invalid File");
        }
    }
}

fn get_orientation(bytes: &Vec<u8>) -> Option<usize> {
    let img_bytes = ImgBytes::new(bytes);
    if !img_bytes.is_jpeg() {
        return None;
    }
    let tag = "112".to_string();
    if let Some(orientation) = img_bytes.exif_tag_value(tag) {
        if orientation.len() == 2 {
            return Some(orientation[1] as usize);
        }
    }
    None
}

struct ImgBytes<'a> {
    bytes: &'a Vec<u8>,
}
impl<'a> ImgBytes<'a> {
    fn new(bytes: &'a Vec<u8>) -> Self {
        Self { bytes }
    }

    fn is_jpeg(&self) -> bool {
        self.hex_str(0, 2) == "ffd8"
    }

    fn hex_str(&self, start: usize, size: usize) -> String {
        let mut hex_str = "".to_string();
        for b in &self.bytes[start..start + size] {
            hex_str.push_str(&format!("{b:0x}"));
        }
        hex_str
    }

    fn as_usize(&self, start: usize, size: usize) -> usize {
        let mut num = 0;
        let bytes_vec = self.bytes[start..start + size].to_vec();
        for (i, b) in bytes_vec.iter().enumerate() {
            num += (*b as u32) << (8 * (size - i - 1));
        }
        num as usize
    }

    fn exif_start_idx(&self) -> Option<usize> {
        for i in 0..self.bytes.len() - 1 {
            if self.hex_str(i, 2) == "ffe1" {
                return Some(i + 10);
            }
        }
        None
    }

    fn exif_tag_value(&self, tag: String) -> Option<Vec<u8>> {
        if let Some(start) = self.exif_start_idx() {
            let offset = self.as_usize(start + 4, 4);
            let ifd0 = start + offset;
            let count = self.as_usize(ifd0, 2);
            for i in 0..count {
                let tag_start = ifd0 + 2 + 12 * i;
                if self.hex_str(tag_start, 2) == tag {
                    let value_size = self.exif_tag_value_size(tag_start);
                    let offset = self.as_usize(tag_start + 8, 4);
                    if value_size <= 4 {
                        let s = tag_start + 8;
                        return Some(self.bytes[s..s + value_size].to_vec());
                    } else {
                        // TODO need to check start position of offset
                        let s = tag_start + 8 + offset;
                        return Some(self.bytes[s..s + value_size].to_vec());
                    }
                }
            }
        }
        None
    }

    fn exif_tag_value_size(&self, tag_start: usize) -> usize {
        let tag_type = self.hex_str(tag_start + 2, 2);
        let count = self.as_usize(tag_start + 4, 4);
        let data_len = match &*tag_type {
            "01" => 1,
            "02" => 1,
            "03" => 2,
            "04" => 4,
            "05" => 8,
            "06" => 1,
            "07" => 1,
            "08" => 2,
            "09" => 4,
            "0A" => 8,
            "0B" => 4,
            "0C" => 8,
            _ => 0,
        };
        data_len * count
    }
}
