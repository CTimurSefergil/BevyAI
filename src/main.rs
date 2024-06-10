use anyhow::Result;
use bevy::prelude::*;
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
        prompt: "A cartoonish style skyscraper".to_string(),
        n: Some(1),
        size: Some("512x512".to_string()),
        response_format: None,
        user: None,
    };
    let rs = openai.image_create(&body);
    let images = rs.unwrap().data.unwrap();
    let image = images.get(0).unwrap();
    if image.url.contains("http") {
        let _ = download_image(&image.url as &str, r#"assets/image.png"#);
    }
}

fn download_image(url: &str, file_name: &str) -> Result<()> {
    let mut response = reqwest::blocking::get(url)?;
    let mut file = File::create(file_name)?;
    copy(&mut response, &mut file)?;

    Ok(())
}
