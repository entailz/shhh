use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, ImageError};
use std::str::FromStr;
use std::io::{self, Read, Write};
use clap::{App, Arg};
use png::{ColorType, Encoder};
use image::io::Reader;

fn main() {
    let matches = App::new("Image Rounder and Shadow Adder")
        .arg(Arg::with_name("input")
            .long("input")
            .short("i")
            .takes_value(true)
            .help("Input image file (default: read from stdin)"))
        .arg(Arg::with_name("output")
            .long("output")
            .short("o")
            .takes_value(true)
            .help("Output image file (default: write to stdout)"))
        .arg(Arg::with_name("corner_radius")
            .long("radius")
            .short("r")
            .takes_value(true)
            .help("Corner radius for rounding"))
        .arg(Arg::with_name("offset")
            .long("offset")
            .short("e")
            .takes_value(true)
            .help("Shadow offset in format x,y"))
        .arg(Arg::with_name("alpha")
            .long("alpha")
            .short("a")
            .takes_value(true)
            .help("Shadow alpha (0-255)"))
        .arg(Arg::with_name("spread")
            .long("spread")
            .short("s")
            .takes_value(true)
            .help("Shadow spread distance"))
        .get_matches();

    let corner_radius = matches.value_of("corner_radius")
        .map(|s| u32::from_str(s).unwrap_or(0))
        .unwrap_or(0);
    let offset = matches.value_of("offset")
        .map(|s| {
            let parts: Vec<&str> = s.split(',').collect();
            if parts.len() == 2 {
                (i32::from_str(parts[0]).unwrap_or(0), i32::from_str(parts[1]).unwrap_or(0))
            } else {
                (0, 0)
            }
        })
        .unwrap_or((0, 0));
    let shadow_alpha = matches.value_of("alpha")
        .map(|s| u8::from_str(s).unwrap_or(128))
        .unwrap_or(128);
    let spread = matches.value_of("spread")
        .map(|s| i32::from_str(s).unwrap_or(10))
        .unwrap_or(10);

    let input_data = if let Some(input_path) = matches.value_of("input") {
        std::fs::read(input_path).expect("Failed to read input file")
    } else {
        let mut buffer = Vec::new();
        match io::stdin().read_to_end(&mut buffer) {
            Ok(0) => {
                eprintln!("Error: No input data received. Make sure you're piping an image to this program.");
                std::process::exit(1);
            }
            Ok(n) => {
                eprintln!("Debug: Read {} bytes from stdin", n);
                buffer
            }
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                std::process::exit(1);
            }
        }
    };

    eprintln!("Debug: Input data size: {} bytes", input_data.len());

    let img = match Reader::new(std::io::Cursor::new(&input_data))
        .with_guessed_format() {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!("Failed to guess image format: {}", e);
                std::process::exit(1);
            }
        };

    eprintln!("Debug: Guessed image format: {:?}", img.format());

    let img = match img.decode() {
        Ok(img) => img,
        Err(e) => {
            match e {
                ImageError::IoError(io_err) => eprintln!("IO Error: {}", io_err),
                ImageError::Unsupported(msg) => eprintln!("Unsupported format: {}", msg),
                _ => eprintln!("Unknown error: {}", e),
            }
            std::process::exit(1);
        }
    };

    eprintln!("Debug: Image successfully decoded");

    let rounded_img = round_corners(&img, corner_radius);

    let result = add_rounded_drop_shadow(&rounded_img, offset.0, offset.1, 10, spread, shadow_alpha)
    .unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

 if let Some(output_path) = matches.value_of("output") {
        result.save(output_path).expect("Failed to save output file");
        eprintln!("Image with rounded corners and drop shadow saved as: {}", output_path);
    } else {
        let rgba_image = result.to_rgba8();
        let (width, height) = rgba_image.dimensions();
        let mut png_data = Vec::new();
        {
            let mut encoder = Encoder::new(&mut png_data, width, height);
            encoder.set_color(ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().expect("Failed to write PNG header");
            writer.write_image_data(rgba_image.as_raw()).expect("Failed to write PNG data");
        }
        io::stdout().lock().write_all(&png_data).expect("Failed to write to stdout");
        io::stdout().flush().expect("Failed to flush stdout");
    }
}

fn add_rounded_drop_shadow(
    rounded_img: &DynamicImage,
    offset_x: i32,
    offset_y: i32,
    blur_radius: u32,
    spread: i32,
    shadow_alpha: u8,
) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let (width, height) = rounded_img.dimensions();

    let total_width = width as i32 + offset_x.abs() + spread as i32 * 2;
    let total_height = height as i32 + offset_y.abs() + spread as i32 * 2;

    let mut output = ImageBuffer::new(total_width as u32, total_height as u32);

    let shadow = create_shadow(rounded_img, blur_radius, spread, shadow_alpha);

    let shadow_x = if offset_x < 0 { 0 } else { offset_x };
    let shadow_y = if offset_y < 0 { 0 } else { offset_y };

    image::imageops::overlay(&mut output, &shadow, shadow_x as i64, shadow_y as i64);

    let image_x = if offset_x < 0 { offset_x.abs() } else { 0 };
    let image_y = if offset_y < 0 { offset_y.abs() } else { 0 };

    image::imageops::overlay(&mut output, rounded_img, image_x as i64, image_y as i64);

    Ok(DynamicImage::ImageRgba8(output))
}

fn round_corners(img: &DynamicImage, radius: u32) -> DynamicImage {
    let (width, height) = img.dimensions();
    let mut rounded = ImageBuffer::new(width, height);
    let radius = radius as f32;

    for (x, y, pixel) in img.to_rgba8().enumerate_pixels() {
        let (dx, dy) = if x < radius as u32 && y < radius as u32 {

            (radius - x as f32, radius - y as f32)
        } else if x >= width - radius as u32 && y < radius as u32 {

            (x as f32 - (width as f32 - radius - 1.0), radius - y as f32)
        } else if x < radius as u32 && y >= height - radius as u32 {

            (radius - x as f32, y as f32 - (height as f32 - radius - 1.0))
        } else if x >= width - radius as u32 && y >= height - radius as u32 {

            (x as f32 - (width as f32 - radius - 1.0), y as f32 - (height as f32 - radius - 1.0))
        } else {

            rounded.put_pixel(x, y, *pixel);
            continue;
        };

        let distance = (dx * dx + dy * dy).sqrt();

        if distance <= radius {
            rounded.put_pixel(x, y, *pixel);
        } else {
            let alpha = ((radius + 1.0 - distance).max(0.0) * 255.0) as u8;
            rounded.put_pixel(x, y, Rgba([pixel[0], pixel[1], pixel[2], alpha.min(pixel[3])]));
        }
    }

    DynamicImage::ImageRgba8(rounded)
}

fn create_shadow(img: &DynamicImage, blur_radius: u32, spread: i32, shadow_alpha: u8) -> DynamicImage {
    let (width, height) = img.dimensions();
    let new_width = (width as i32 + spread.abs() * 2 + blur_radius as i32 * 2) as u32;
    let new_height = (height as i32 + spread.abs() * 2 + blur_radius as i32 * 2) as u32;
    let mut shadow = ImageBuffer::new(new_width, new_height);

    let spread_img = if spread > 0 {
        image::imageops::resize(img, width + spread as u32 * 2, height + spread as u32 * 2, image::imageops::FilterType::Nearest)
    } else if spread < 0 {
        image::imageops::resize(img, (width as i32 + spread * 2).max(1) as u32, (height as i32 + spread * 2).max(1) as u32, image::imageops::FilterType::Nearest)
    } else {
        img.to_rgba8()
    };

    let offset = (spread.abs() + blur_radius as i32) as u32;
    image::imageops::overlay(&mut shadow, &spread_img, offset.into(), offset.into());

    for (x, y, pixel) in shadow.enumerate_pixels_mut() {
        let alpha = pixel[3] as f32 / 255.0;
        let distance_to_edge = ((x as i32 - new_width as i32 / 2).abs().min((y as i32 - new_height as i32 / 2).abs()) as f32) / (new_width.min(new_height) as f32 / 2.0);
        let fade_factor = (1.0 - distance_to_edge).powi(2);
        let new_alpha = alpha * shadow_alpha as f32 * fade_factor;
        pixel[0] = 0;
        pixel[1] = 0;
        pixel[2] = 0;
        pixel[3] = new_alpha as u8;
    }

    let blurred = image::imageops::blur(&shadow, blur_radius as f32 * 2.0);
    DynamicImage::ImageRgba8(blurred)
}
