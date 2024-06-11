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
        .add_systems(Startup, image_create)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn image_create() {
    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

    let body = ImagesBody {
        prompt: "Draw a cartoonish style 16 bit female wizard".to_string(),
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

// NEDEN ÇALIŞMIYORSUN !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
fn convert_to_transparent(path: &str) {
    let img = image::open(path).unwrap();
    let mut file = File::create(format!(r#"assets/image1.png"#)).unwrap();
    copy(&mut img.as_bytes(), &mut file).unwrap();

    let mut img = img.to_rgba32f();
    for p in img.pixels_mut() {
        if p[0] == 255.0 && p[1] == 255.0 && p[2] == 255.0 {
            p[3] = 0.0;
        }
    }

    std::io::copy(&mut img.as_bytes(), &mut file).unwrap();
}
