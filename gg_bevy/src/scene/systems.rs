use bevy::prelude::*;

use crate::coords::world_dimensions;
use crate::resources::{ConfigResource, GameStateResource, HeatmapResource};
use crate::scene::{GroundPlane, RewardText, WALL_HEIGHT, WallBundle, WallGraphicsAssets};

/// Spawns the upper-right HUD panel: generation seed (or the world file
/// path, if one was supplied — a procedural seed wouldn't mean anything for
/// a fixed board), episode seed, the live current-state reward, and the
/// control-hint lines. All one panel so nothing has to be manually
/// positioned to avoid overlapping a second panel.
pub fn setup_key_instructions(
    mut commands: Commands,
    config: Res<ConfigResource>,
    state: Res<GameStateResource>,
    heatmap: Res<HeatmapResource>,
) {
    let config = &config.0;
    let is_teleop = config.agent.ghost_policy == Some(gg_core::GhostPolicy::Teleop);
    let has_value_heatmap = heatmap.0.is_some();
    let ghost_occlusion_enabled = config.agent.ghost_occlusion;

    let generation_line = match &config.world_file {
        Some(path) => format!("World: {}", path),
        None => format!(
            "Generation Seed: {}",
            config
                .generation_seed
                .expect("Should have generated a generation seed before spawning the HUD")
        ),
    };
    let episode_seed = config
        .episode_seed
        .expect("Should have generated an episode seed before spawning the HUD");

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
                Text::new(generation_line),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextLayout::new_with_justify(Justify::Right),
            ));
            parent.spawn((
                Text::new(format!("Episode Seed: {}", episode_seed)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextLayout::new_with_justify(Justify::Right),
            ));
            parent.spawn((
                Text::new(format!("Reward: {}", state.0.reward)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextLayout::new_with_justify(Justify::Right),
                RewardText,
            ));
            parent.spawn((
                Text::new("+/-: Zoom In/Out | Shift+Drag: Pan Camera"),
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
            parent.spawn((
                Text::new("V: Toggle Value Heatmap"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if has_value_heatmap {
                    Color::WHITE
                } else {
                    Color::srgb(0.7, 0.7, 0.7)
                }),
                TextLayout::new_with_justify(Justify::Right),
            ));
            if has_value_heatmap {
                parent
                    .spawn(Node {
                        display: Display::Flex,
                        justify_content: JustifyContent::FlexEnd,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|legend| {
                        let label_font = TextFont {
                            font_size: 11.0,
                            ..default()
                        };
                        legend.spawn((
                            Text::new("negative"),
                            label_font.clone(),
                            TextColor(Color::srgb(0.267, 0.005, 0.329)),
                        ));
                        legend.spawn((
                            Text::new("zero"),
                            label_font.clone(),
                            TextColor(Color::srgb(0.129, 0.567, 0.551)),
                        ));
                        legend.spawn((
                            Text::new("positive"),
                            label_font,
                            TextColor(Color::srgb(0.993, 0.906, 0.144)),
                        ));
                    });
            }
            parent.spawn((
                Text::new("F: Toggle Agent Visibility Overlay"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if ghost_occlusion_enabled {
                    Color::srgb(0.7, 0.7, 0.7)
                } else {
                    Color::WHITE
                }),
                TextLayout::new_with_justify(Justify::Right),
            ));

            if is_teleop {
                parent.spawn((
                    Text::new(
                        "Arrows/WASD: Move Ghost | Space: Ghost Stays (advances one step)",
                    ),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextLayout::new_with_justify(Justify::Right),
                ));
            } else {
                parent.spawn((
                    Text::new("Space: Pause/Play"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextLayout::new_with_justify(Justify::Right),
                ));
            }
        });
}

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    config: Res<ConfigResource>,
    state: Res<GameStateResource>,
) {
    let cell = config.0.world_generation.cell_size;
    let (world_width, world_height) =
        world_dimensions(state.0.board.width, state.0.board.height, cell);

    let mut entity = commands.spawn((
        Name::new("Ground Plane"),
        GroundPlane,
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(world_width, 1.0, world_height)),
    ));

    if let (Some(meshes), Some(materials)) = (&mut meshes, &mut materials) {
        let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let material = materials.add(Color::srgb_u8(0, 140, 0));
        entity.insert((Mesh3d(mesh), MeshMaterial3d(material)));
    }
}

/// Keeps the HUD's "Reward: ..." line current — the only line in that panel
/// that changes after Startup (seeds/world path are fixed for the session).
pub fn update_reward_text(
    state: Res<GameStateResource>,
    mut query: Query<&mut Text, With<RewardText>>,
) {
    for mut text in &mut query {
        text.0 = format!("Reward: {}", state.0.reward);
    }
}

/// Spawns wall meshes from the already-resolved `GameStateResource` (never
/// regenerates the board — that happens once, outside Bevy, before
/// `build_app` is called, whether procedurally or from a loaded world file).
pub fn spawn_walls(
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    graphics: Option<Res<WallGraphicsAssets>>,
    config: Res<ConfigResource>,
    state: Res<GameStateResource>,
) {
    let config = &config.0;
    let board = &state.0.board;

    if !config.headless {
        let cell = config.world_generation.cell_size;
        let (world_width, world_height) = world_dimensions(board.width, board.height, cell);

        for (col, row) in &board.wall_positions {
            let p0 = (
                (*col as f32 + 0.5) * cell - (world_width * 0.5),
                (*row as f32 + 0.5) * cell - (world_height * 0.5),
            );

            let mut entity = commands.spawn(WallBundle::new(p0.into(), p0.into()));

            if let (Some(meshes), Some(graphics)) = (&mut meshes, &graphics) {
                // Square footprint the size of a cell; height = WALL_HEIGHT
                let mesh = meshes.add(Cuboid::new(cell, WALL_HEIGHT, cell));
                entity.insert((Mesh3d(mesh), MeshMaterial3d(graphics.material.clone())));
            }
        }
    }
}
