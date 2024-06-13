use anyhow::Result;
use bevy::prelude::*;
use image::*;
use openai_api_rust::images::*;
use openai_api_rust::*;
use std::{fs::File, io::copy};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Startup, create_image)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn create_image() {
    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

    let body = ImagesBody {
        prompt: "Draw a cartoonish style 16 bit female wizard. And make sure that the background is pure white"
            .to_string(),
        n: Some(1),
        size: Some("256x256".to_string()),
        response_format: None,
        user: None,
    };
    let rs = openai.image_create(&body);
    let images = rs.unwrap().data.unwrap();
    let image = images.get(0).unwrap();
    if image.url.contains("http") {
        let _ = download_and_save_image(&image.url as &str, r#"assets/image.png"#);
        convert_to_transparent(r#"assets/image.png"#);
    }
}

fn download_and_save_image(url: &str, file_name: &str) -> Result<()> {
    let mut response = reqwest::blocking::get(url)?;
    let mut file = File::create(file_name)?;
    copy(&mut response, &mut file)?;

    Ok(())
}

// ARKA PLAN TAMAMEN BEYAZ OLMADIĞI İÇİN ÇALIŞMIYOR
fn convert_to_transparent(path: &str) {
    let mut image = image::open(path).unwrap();
    let (width, height) = image.dimensions();
    for a in 200..=255 {
        for x in 0..width {
            for y in 0..height {
                let pixel = image.get_pixel(x, y);
                if pixel[0] == a && pixel[1] == a && pixel[2] == a {
                    image.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
                }
            }
        }
    }
    image.save(r#"assets/image1.png"#).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_convert_to_transparent() {
        convert_to_transparent(r#"assets/image.png"#);
    }
}
