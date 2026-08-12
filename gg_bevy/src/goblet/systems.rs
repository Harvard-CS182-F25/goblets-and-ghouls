use bevy::prelude::*;
use gg_core::Goblet as GobletData;

use crate::coords::{cell_to_world, world_dimensions};
use crate::goblet::{GobletBundle, GobletRewardLabel};
use crate::resources::{ConfigResource, GameStateResource};
use crate::scene::{GOBLET_HEIGHT, RenderMeshAssets};

use super::visual::GobletGraphicsAssets;

const LABEL_FILL: f32 = 0.80;
const DIGIT_WIDTH_RATIO: f32 = 0.6;

pub fn spawn_goblets(
    mut commands: Commands,
    meshes: Option<Res<RenderMeshAssets>>,
    goblet_graphics: Option<Res<GobletGraphicsAssets>>,
    state: Res<GameStateResource>,
    config: Res<ConfigResource>,
) {
    let cell = config.0.world_generation.cell_size;
    let (world_width, world_height) =
        world_dimensions(state.0.board.width, state.0.board.height, cell);

    for (i, &GobletData { position, reward }) in state.0.board.goblets.iter().enumerate() {
        let goblet_name = format!("Goblet {}", i + 1);
        let world_position = cell_to_world(
            position,
            cell,
            world_width,
            world_height,
            GOBLET_HEIGHT / 2.0,
        );

        let mut entity = commands.spawn(GobletBundle::new(&goblet_name, world_position, reward));

        if let Some(goblet_graphics) = &goblet_graphics
            && let Some(meshes) = &meshes
        {
            let material = if reward > 0 {
                goblet_graphics.material.clone()
            } else {
                goblet_graphics.false_material.clone()
            };

            entity.insert((Mesh3d(meshes.goblet.clone()), MeshMaterial3d(material)));

            let label = reward.unsigned_abs().to_string();
            let (label_color, shadow_color) = if reward > 0 {
                (
                    Color::srgba(0.4, 0.25, 0.0, 0.6),
                    Color::srgba(0.4, 0.25, 0.0, 0.4),
                )
            } else {
                (
                    Color::srgba(0.45, 0.05, 0.1, 0.6),
                    Color::srgba(0.45, 0.05, 0.1, 0.4),
                )
            };
            commands.spawn((
                GobletRewardLabel {
                    position,
                    digit_count: label.len(),
                },
                Text::new(label),
                TextFont::default(),
                TextColor(label_color),
                TextShadow {
                    offset: Vec2::splat(0.5),
                    color: shadow_color,
                },
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                Visibility::Hidden,
            ));
        }
    }
}

pub fn update_goblet_reward_labels(
    config: Res<ConfigResource>,
    state: Res<GameStateResource>,
    camera_q: Query<(&Camera, &GlobalTransform, &Projection), With<Camera3d>>,
    mut labels: Query<(
        &GobletRewardLabel,
        &mut TextFont,
        &mut Node,
        &mut Visibility,
    )>,
) {
    let Ok((camera, camera_transform, Projection::Orthographic(orthographic))) = camera_q.single()
    else {
        return;
    };

    let cell_size = config.0.world_generation.cell_size;
    let (world_width, world_height) =
        world_dimensions(state.0.board.width, state.0.board.height, cell_size);
    let label_size = cell_size / orthographic.scale.abs() * 0.8;

    for (label, mut font, mut node, mut visibility) in &mut labels {
        let world_position = cell_to_world(
            label.position,
            cell_size,
            world_width,
            world_height,
            GOBLET_HEIGHT,
        );

        match camera.world_to_viewport(camera_transform, world_position) {
            Ok(screen_position) => {
                let text_aspect_ratio = DIGIT_WIDTH_RATIO * label.digit_count as f32;
                let font_size = label_size * LABEL_FILL / text_aspect_ratio.max(1.0);
                font.font_size = font_size;
                node.width = Val::Px(label_size);
                node.left = Val::Px(screen_position.x - label_size / 2.0);
                node.top = Val::Px(screen_position.y - font_size * 0.6);
                *visibility = Visibility::Visible;
            }
            Err(_) => *visibility = Visibility::Hidden,
        }
    }
}
