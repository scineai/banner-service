mod config;
mod models;

use std::{fs, io::Cursor};

use clap::Parser;
use image::{
    imageops,
    io::Reader as ImageReader,
};
use rayon::prelude::*;
use rusttype::{Font, Scale};
use indicatif::{ProgressBar, ProgressIterator};

use config::Config;
use models::UnsplashResponse;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub query: String,
    pub text: String,
    pub font_size: u32,
    pub border_radius: u32,
    pub description: String,
    pub description_color_offset: u32
}

fn main() {
    let config = Config::build();
    let args = Args::parse();
    let progress = ProgressBar::new_spinner();
    progress.set_message("Fetching images");

    /*
        Steps:
        1) Fetch images from unsplash
        2) Download images
        3) Process images by first resizing them accordingly
        4) Then we can add the text on
    */

    // Step 1: Fetch images from unsplash
    let url = format!(
        "https://api.unsplash.com/search/photos?query={}",
        args.query
    );
    let auth_header = format!("Client-ID {}", config.access_key);
    let response = ureq::get(&url)
        .set("Authorization", &auth_header)
        .call()
        .expect("Failed to fetch images from unsplash");

    let data: UnsplashResponse = response.into_json().unwrap();
    progress.finish();

    // first attempt to delete the current .stage/ directory if it exists
    fs::remove_dir_all(".stage/").ok();

    // create a directory where processing can be isolated to
    fs::create_dir(".stage/").unwrap();

    println!("Downloading Images:");
    for (i, result) in data.results.iter().progress().enumerate() {
        let raw_url = &result.urls.raw;
        let response = ureq::get(&raw_url).call().unwrap();
        let mut body: Vec<u8> = Vec::new();
        response.into_reader().read_to_end(&mut body).unwrap();

        let filename = format!(".stage/download-{i}.png");
        fs::write(filename, body).unwrap();
    }

    // now that we have our images downloaded into the stage folder
    // we can now begin the process of manipulating the images
    println!("Processing Images (bar may take a few seconds to appear):");
    let process_progress_bar = ProgressBar::new(data.results.len() as u64);
    process_progress_bar.tick();
    fs::read_dir(".stage/").unwrap().par_bridge().map(|entry| {
        if let Ok(entry) = entry {
            let image = fs::read(entry.path()).expect("Failed to read downloaded image");
            let bar = process_progress_bar.clone();
            process_image(image, &bar, &args.text, args.font_size, args.border_radius, args.description_color_offset, &args.description);
        }

        0
    }).collect::<Vec<u32>>();

    println!("[ ✅ ] Complete!");
}

fn process_image(image: Vec<u8>, bar: &ProgressBar, text: &String, font_size: u32, bradius: u32, doffset: u32, dtext: &String) {
    /*
    (- first step is to load the image but
    we won't include this as part of the logic outline)
    1) resize the image
    2) apply a filter to the image to make it darker and reduce brightness
    3) add white text onto the image at the center
    */
    let cursor = Cursor::new(image);
    let reader = ImageReader::new(cursor);
    let image_loaded = reader.with_guessed_format().unwrap().decode().unwrap();

    // Test dimensions: 2251x432
    // resize image
    let id = uuid::Uuid::new_v4().to_string();
    let save_path = format!(".stage/final-{id}.png");

    let resized_image = image_loaded.resize_exact(
        2251,
        432,
        imageops::Nearest
    );
    let brightened_image = resized_image.brighten(-85);
    let blurred_image = brightened_image.blur(15.0);

    // draw text
    let font_data = include_bytes!("Roboto-Bold.ttf");
    let font_data_vec = font_data.to_vec();
    let font = Font::try_from_bytes(&font_data_vec).unwrap();
    let font_size_scale = Scale::uniform(font_size as f32);

    let (text_size_x, text_size_y) = imageproc::drawing::text_size(font_size_scale, &font, text);
    let (image_size_x, image_size_y) = (blurred_image.width() as i32, blurred_image.height() as i32);

    let location = (
        (image_size_x - text_size_x) / 2,
        (image_size_y - text_size_y) / 2
    );
    let (location_x, location_y) = location;
    let location_x = location_x / 7;
    let color = image::Rgba([255, 255, 255, 255]);
    let final_image = imageproc::drawing::draw_text(&blurred_image, color, location_x, location_y, font_size_scale, &font, text);

    // add the description
    let description_text_font_size = font_size / 2;
    let description_text_font_size_scale = Scale::uniform(description_text_font_size as f32);
    let (_, description_text_size_y) = imageproc::drawing::text_size(description_text_font_size_scale, &font, dtext);
    let offseted = 255 - doffset;
    let offseted = offseted as u8;
    let description_color = image::Rgba([offseted, offseted, offseted, 255]);
    let (description_location_x, description_location_y) = location;
    let description_location_x = description_location_x / 7;
    let description_location_y = description_location_y + description_text_size_y + 30;
    let mut final_image = imageproc::drawing::draw_text(&final_image, description_color, description_location_x, description_location_y, description_text_font_size_scale, &font, dtext);

    // add some rounded corners to the final image
    // top left
    border_radius(&mut final_image, bradius, |x, y| (x - 1, y - 1));
    // top right
    border_radius(&mut final_image, bradius, |x, y| (image_size_x as u32 - x, y - 1));
    // bottom right
    border_radius(&mut final_image, bradius, |x, y| (image_size_x as u32 - x, image_size_y as u32 - y));
    // bottom left
    border_radius(&mut final_image, bradius, |x, y| (x - 1, image_size_y as u32 - y));

    final_image.save(save_path).unwrap();

    bar.inc(1);
}

// https://github.com/steffahn/og_image_writer
fn border_radius(
    img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    r: u32,
    coordinates: impl Fn(u32, u32) -> (u32, u32),
) {
    if r == 0 {
        return;
    }
    let r0 = r;

    // 16x antialiasing: 16x16 grid creates 256 possible shades, great for u8!
    let r = 16 * r;

    let mut x = 0;
    let mut y = r - 1;
    let mut p: i32 = 2 - r as i32;

    // ...

    let mut alpha: u16 = 0;
    let mut skip_draw = true;

    let draw = |img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, alpha, x, y| {
        debug_assert!((1..=256).contains(&alpha));
        let pixel_alpha = &mut img[coordinates(r0 - x, r0 - y)].0[3];
        *pixel_alpha = ((alpha * *pixel_alpha as u16 + 128) / 256) as u8;
    };

    'l: loop {
        // (comments for bottom_right case:)
        // remove contents below current position
        {
            let i = x / 16;
            for j in y / 16 + 1..r0 {
                img[coordinates(r0 - i, r0 - j)].0[3] = 0;
            }
        }
        // remove contents right of current position mirrored
        {
            let j = x / 16;
            for i in y / 16 + 1..r0 {
                img[coordinates(r0 - i, r0 - j)].0[3] = 0;
            }
        }

        // draw when moving to next pixel in x-direction
        if !skip_draw {
            draw(img, alpha, x / 16 - 1, y / 16);
            draw(img, alpha, y / 16, x / 16 - 1);
            alpha = 0;
        }

        for _ in 0..16 {
            skip_draw = false;

            if x >= y {
                break 'l;
            }

            alpha += y as u16 % 16 + 1;
            if p < 0 {
                x += 1;
                p += (2 * x + 2) as i32;
            } else {
                // draw when moving to next pixel in y-direction
                if y % 16 == 0 {
                    draw(img, alpha, x / 16, y / 16);
                    draw(img, alpha, y / 16, x / 16);
                    skip_draw = true;
                    alpha = (x + 1) as u16 % 16 * 16;
                }

                x += 1;
                p -= (2 * (y - x) + 2) as i32;
                y -= 1;
            }
        }
    }

    // one corner pixel left
    if x / 16 == y / 16 {
        // column under current position possibly not yet accounted
        if x == y {
            alpha += y as u16 % 16 + 1;
        }
        let s = y as u16 % 16 + 1;
        let alpha = 2 * alpha - s * s;
        draw(img, alpha, x / 16, y / 16);
    }

    // remove remaining square of content in the corner
    let range = y / 16 + 1..r0;
    for i in range.clone() {
        for j in range.clone() {
            img[coordinates(r0 - i, r0 - j)].0[3] = 0;
        }
    }
}
