use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;

use crate::{
    core::GGConfig,
    game_state::{Board, GameState},
    scene::{GroundPlane, WALL_HEIGHT, WallBundle, WallGraphicsAssets},
};

pub fn setup_key_instructions(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::Grid,
                top: Val::Px(5.0),
                right: Val::Px(5.0),
                padding: Val::Px(2.5).into(),
                justify_items: JustifyItems::End,
                align_items: AlignItems::Start,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("+/-: Zoom In/Out | Arrow Keys: Pan Camera"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextLayout::new_with_justify(Justify::Right),
            ));
            parent.spawn((
                Text::new("P: Toggle Policy Visualization"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextLayout::new_with_justify(Justify::Right),
            ));
        });
}

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    config: Res<GGConfig>,
) {
    let mut entity = commands.spawn((
        Name::new("Ground Plane"),
        GroundPlane,
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(
            config.world_generation.world_width,
            1.0,
            config.world_generation.world_height,
        )),
    ));

    if let (Some(meshes), Some(materials)) = (&mut meshes, &mut materials) {
        let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let material = materials.add(Color::srgb_u8(0, 140, 0));
        entity.insert((Mesh3d(mesh), MeshMaterial3d(material)));
    }
}

pub fn spawn_seed_text(mut commands: Commands, config: Res<GGConfig>) {
    let seed = config
        .generation_seed
        .expect("Should have generated a seed before spawning seed text");

    if config.headless {
        return;
    }

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::Grid,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                padding: Val::Px(2.5).into(),
                justify_items: JustifyItems::Start,
                align_items: AlignItems::Start,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format! {"Seed: {}", seed}),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextLayout::new_with_justify(Justify::Right),
            ));
        });
}

pub fn spawn_walls(
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    graphics: Option<Res<WallGraphicsAssets>>,
    config: Res<GGConfig>,
) {
    let game_state: GameState = Board::new(&mut rng, &config).into();

    if !config.headless {
        let cell = config.world_generation.cell_size;

        for (col, row) in &game_state.board.wall_positions {
            let p0 = (
                (*col as f32 + 0.5) * cell - (config.world_generation.world_width * 0.5),
                (*row as f32 + 0.5) * cell - (config.world_generation.world_height * 0.5),
            );

            let mut entity = commands.spawn(WallBundle::new(p0.into(), p0.into()));

            if let (Some(meshes), Some(graphics)) = (&mut meshes, &graphics) {
                // Square footprint the size of a cell; height = WALL_HEIGHT
                let mesh = meshes.add(Cuboid::new(cell, WALL_HEIGHT, cell));
                entity.insert((Mesh3d(mesh), MeshMaterial3d(graphics.material.clone())));
            }
        }
    }

    commands.insert_resource(game_state);
}
