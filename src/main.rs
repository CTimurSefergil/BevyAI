use anyhow::Result;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
    utils::hashbrown::HashSet,
    window::close_on_esc,
};
use bevy_pancam::{PanCam, PanCamPlugin};
use image::*;
use noise::{NoiseFn, Perlin, Seedable};
use openai_api_rust::images::*;
use openai_api_rust::*;
use rand::Rng;
use std::{fs::File, io::copy, process::Command};

const GRASS_PATH: &str = "grass_transparent.png";
const WATER_PATH: &str = "water_transparent.png";
const SPRITE_SCALE_FACTOR: usize = 1;

const GRID_COLS: usize = 500;
const GRID_ROWS: usize = 250;
const BG_COLOR: (u8, u8, u8) = (0, 187, 163);

const TILE_W: usize = 256;
const TILE_H: usize = 256;

const NOISE_SCALE: f64 = 0.01;

const MOVEMENT_SPEED: f32 = 840.0;
const CAM_LERP_FACTOR: f32 = 2.;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PanCamPlugin)
        .insert_resource(ClearColor(Color::rgba_u8(
            BG_COLOR.0, BG_COLOR.1, BG_COLOR.2, 255,
        )))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (handle_player_movement_input, update_camera).chain(),
        )
        .add_systems(Update, close_on_esc)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    /*create_image(
        "Create a 16 bit female witch character with white blackground",
        "grass",
    );
    create_image("Create a cartoonish water texture", "water");
    create_image(
        "Create a single female warrior character with white blackground",
        "character",
    );
    */
    commands
        .spawn(Camera2dBundle::default())
        .insert(PanCam::default());

    let mut rng = rand::thread_rng();
    let perlin = Perlin::new(rng.gen());
    let mut grass_tiles = HashSet::new();
    let mut water_tiles = HashSet::new();
    for x in 0..GRID_COLS {
        for y in 0..GRID_ROWS {
            let val = perlin.get([x as f64 * NOISE_SCALE, y as f64 * NOISE_SCALE]);
            if val > 0.2 {
                water_tiles.insert((x, y));
                continue;
            }
            grass_tiles.insert((x, y));
        }
    }

    for (x, y) in grass_tiles.iter() {
        let (x, y) = grid_to_world(*x as f32, *y as f32);
        commands.spawn(SpriteSheetBundle {
            sprite: Sprite::default(),
            transform: Transform::from_scale(Vec3::splat(SPRITE_SCALE_FACTOR as f32))
                .with_translation(Vec3 {
                    x: (x),
                    y: (y),
                    z: (0.0),
                }),
            texture: asset_server.load(GRASS_PATH),
            ..default()
        });
    }

    for (x, y) in water_tiles.iter() {
        let (x, y) = grid_to_world(*x as f32, *y as f32);
        commands.spawn(SpriteSheetBundle {
            sprite: Sprite::default(),
            transform: Transform::from_scale(Vec3::splat(SPRITE_SCALE_FACTOR as f32))
                .with_translation(Vec3 {
                    x: (x),
                    y: (y),
                    z: (0.0),
                }),
            texture: asset_server.load(WATER_PATH),
            ..default()
        });
    }

    commands.spawn((
        Name::new("Player"),
        Player,
        SpriteSheetBundle {
            sprite: Sprite::default(),
            transform: Transform::from_scale(Vec3::splat(16.0 as f32)).with_translation(Vec3 {
                x: (100.0),
                y: (100.0),
                z: (0.0),
            }),
            texture: asset_server.load("character_transparent.png"),
            ..default()
        },
    ));
}

fn grid_to_world(x: f32, y: f32) -> (f32, f32) {
    (
        x * TILE_W as f32 * SPRITE_SCALE_FACTOR as f32,
        y * TILE_H as f32 * SPRITE_SCALE_FACTOR as f32,
    )
}

fn create_image(prompt: &str, path_name: &str) {
    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

    let body = ImagesBody {
        prompt: prompt.to_string(),
        n: Some(1),
        size: Some("256x256".to_string()),
        response_format: None,
        user: None,
    };
    let rs = openai.image_create(&body);
    let images = rs.unwrap().data.unwrap();
    let image = images.get(0).unwrap();
    if image.url.contains("http") {
        let _ = download_and_save_image(
            &image.url as &str,
            r#"assets/"#.to_owned() + path_name + ".png",
        );
        convert_to_transparent(
            (r#"assets/"#.to_owned() + path_name + ".png").as_str(),
            (r#"assets/"#.to_owned() + path_name).as_str(),
        );
    }
}

fn download_and_save_image(url: &str, file_name: String) -> Result<()> {
    let mut response = reqwest::blocking::get(url)?;
    let mut file = File::create(file_name)?;
    copy(&mut response, &mut file)?;

    Ok(())
}

fn convert_to_transparent(path: &str, path1: &str) {
    let save_path = path1.to_owned() + "_transparent" + ".png";
    let image = image::open(path).unwrap();
    let mut rgbaimage = image.to_rgba8();
    let (width, height) = image.dimensions();
    for a in 230..=255 {
        for b in 230..=255 {
            for c in 230..=255 {
                for x in 0..width {
                    for y in 0..height {
                        let pixel = image.get_pixel(x, y);
                        if pixel[0] == a && pixel[1] == b && pixel[2] == c {
                            rgbaimage.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
                        }
                    }
                }
            }
        }
    }
    rgbaimage.save(save_path).unwrap();
}

fn handle_player_movement_input(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut Sprite), With<Player>>,
) {
    let mut intent = Vec3::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    let intent = intent.normalize_or_zero();
    let target_velocity = intent * MOVEMENT_SPEED;

    for (mut transform, mut sprite) in &mut player_query {
        transform.translation += target_velocity * time.delta_seconds();
        if intent.x != 0.0 {
            sprite.flip_x = intent.x < 0.0;
        }
    }
}

fn update_camera(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };

    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    camera.translation = camera
        .translation
        .lerp(direction, time.delta_seconds() * CAM_LERP_FACTOR);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_convert_to_transparent() {
        convert_to_transparent("image.png", "image");
    }
}
