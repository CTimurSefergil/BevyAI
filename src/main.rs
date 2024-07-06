use anyhow::Result;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    utils::hashbrown::HashSet,
    window::close_on_esc,
};
use bevy_pancam::{PanCam, PanCamPlugin};
use image::*;
use noise::{NoiseFn, Perlin, Seedable};
use openai_api_rust::images::*;
use openai_api_rust::*;
use rand::Rng;
use std::{fs::File, io::copy};

const SPRITE_SHEET_PATH: &str = "image1.png";
const SPRITE_SCALE_FACTOR: usize = 5;

const GRID_COLS: usize = 200;
const GRID_ROWS: usize = 100;

const NOISE_SCALE: f64 = 0.07;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PanCamPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, create_image)
        .add_systems(Update, close_on_esc)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(PanCam::default());

    let mut rng = rand::thread_rng();
    let perlin = Perlin::new(rng.gen());
    let mut tiles = HashSet::new();
    for x in 0..GRID_COLS {
        for y in 0..GRID_ROWS {
            let val = perlin.get([x as f64 * NOISE_SCALE, y as f64 * NOISE_SCALE]);
            if val > 0.2 {
                continue;
            }
            tiles.insert((x, y));
        }
    }

    for (x, y) in tiles.iter() {
        let (x, y) = grid_to_world(*x as f32, *y as f32);
        commands.spawn(SpriteSheetBundle {
            sprite: Sprite::default(),
            transform: Transform::from_scale(Vec3::splat(SPRITE_SCALE_FACTOR as f32))
                .with_translation(Vec3 {
                    x: (x),
                    y: (y),
                    z: (0.0),
                }),
            texture: asset_server.load(SPRITE_SHEET_PATH),
            ..default()
        });
    }
}

fn grid_to_world(x: f32, y: f32) -> (f32, f32) {
    (
        x * TILE_W as f32 * SPRITE_SCALE_FACTOR as f32,
        y * TILE_H as f32 * SPRITE_SCALE_FACTOR as f32,
    )
}

fn create_image() {
    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

    let body = ImagesBody {
        prompt: "Draw a cartoonish style 16 bit female wizard. And make sure that the background is pure white"
            .to_string(),
        n: Some(1),
        size: Some("16x16".to_string()),
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

fn convert_to_transparent(path: &str) {
    let mut image = image::open(path).unwrap();
    let (width, height) = image.dimensions();
    for a in 230..=255 {
        for b in 230..=255 {
            for c in 230..=255 {
                for x in 0..width {
                    for y in 0..height {
                        let pixel = image.get_pixel(x, y);
                        if pixel[0] == a && pixel[1] == b && pixel[2] == c {
                            image.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
                        }
                    }
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
