use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    utils::hashbrown::HashSet,
    window::close_on_esc,
};
use bevy_pancam::{PanCam, PanCamPlugin};
use noise::{NoiseFn, Perlin, Seedable};
use rand::Rng;

const SPRITE_SHEET_PATH: &str = "kitchen-sink.png";
const SPRITE_SCALE_FACTOR: usize = 5;
const TILE_W: usize = 6;
const TILE_H: usize = 8;

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
        .add_systems(Update, close_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(PanCam::default());

    let texture_handle: Handle<Image> = asset_server.load(SPRITE_SHEET_PATH);
    let texture_atlas =
        TextureAtlasLayout::from_grid(Vec2::new(TILE_W as f32, TILE_H as f32), 7, 1, None, None);
    let layout_handle = texture_atlases.add(texture_atlas);

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
            atlas: TextureAtlas {
                layout: layout_handle.clone(),
                index: 5,
            },
            transform: Transform::from_scale(Vec3::splat(SPRITE_SCALE_FACTOR as f32))
                .with_translation(Vec3 {
                    x: (x),
                    y: (y),
                    z: (0.0),
                }),
            texture: texture_handle.clone(),
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
//yarrockdickcock
