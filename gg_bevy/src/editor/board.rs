use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

use crate::agent::AgentGraphicsAssets;
use crate::coords::{cell_to_world, world_dimensions};
use crate::goblet::GobletGraphicsAssets;
use crate::scene::{AGENT_HEIGHT, GHOST_HEIGHT, GOBLET_HEIGHT, WALL_HEIGHT, GroundPlane, WallGraphicsAssets};

use super::{CELL_SIZE, PANEL_WIDTH, EditorBoard, EditorState, EditorTile};

const GRID_LINE_WIDTH: f32 = 0.2;
const GRID_LINE_HEIGHT: f32 = 0.02;
/// Hide grid lines once individual cells are too small.
const MIN_GRID_CELL_SIZE_PX: f32 = 20.0;

/// Marker: the tile entity at board position `(row, col)`.
#[derive(Component)]
pub struct EditorTileEntity {
    pub row: usize,
    pub col: usize,
}

/// Marker: a persistent grid-line overlay entity.
#[derive(Component)]
pub struct GridLine;

/// Meshes shared by all editor tiles of the same kind.
#[derive(Resource)]
pub(crate) struct EditorMeshAssets {
    wall: Handle<Mesh>,
    agent: Handle<Mesh>,
    ghost: Handle<Mesh>,
    goblet: Handle<Mesh>,
}

/// Must run in `PreStartup` — `setup_tiles` (in `Startup`) needs these
/// resources to already exist, and `Commands`-queued inserts aren't applied
/// until the end of the schedule they're queued in.
pub fn init_visual_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.init_resource::<WallGraphicsAssets>();
    commands.init_resource::<AgentGraphicsAssets>();
    commands.init_resource::<GobletGraphicsAssets>();
    commands.insert_resource(EditorMeshAssets {
        wall: meshes.add(Cuboid::new(CELL_SIZE, WALL_HEIGHT, CELL_SIZE)),
        agent: meshes.add(Cuboid::new(CELL_SIZE, AGENT_HEIGHT, CELL_SIZE)),
        ghost: meshes.add(Cuboid::new(CELL_SIZE, GHOST_HEIGHT, CELL_SIZE)),
        goblet: meshes.add(Cylinder::new(CELL_SIZE / 2.0, GOBLET_HEIGHT)),
    });
}

pub fn setup_scene(
    mut commands: Commands,
    board: Res<EditorBoard>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    windows: Query<&Window>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1_500.0,
        ..Default::default()
    });
    commands.insert_resource(ClearColor(Color::srgb_u8(0, 136, 255)));

    let (world_width, world_height) = world_dimensions(board.width, board.height, CELL_SIZE);
    let (scale, x_offset) = windows
        .single()
        .map(|window| {
            let visible_width = (window.width() - PANEL_WIDTH).max(1.0);
            let scale = (world_width * 1.15 / visible_width)
                .max(world_height * 1.15 / window.height());
            (-scale, scale * PANEL_WIDTH / 2.0)
        })
        .unwrap_or((-0.15, 0.0));

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(x_offset, 10.0, 0.0))
            .looking_at(Vec3::new(x_offset, 0.0, 0.0), Vec3::NEG_Z),
        Projection::from(OrthographicProjection {
            scale,
            ..OrthographicProjection::default_3d()
        }),
    ));

    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let material = materials.add(Color::srgb_u8(0, 140, 0));

    commands.spawn((
        Name::new("Ground Plane"),
        GroundPlane,
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(world_width, 1.0, world_height)),
        Mesh3d(mesh),
        MeshMaterial3d(material),
    ));
}

/// Spawns grid lines once so they remain a continuous overlay above board
/// objects, rather than being occluded as ground-level gizmos.
pub fn setup_grid(
    mut commands: Commands,
    board: Res<EditorBoard>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (world_width, world_height) = world_dimensions(board.width, board.height, CELL_SIZE);
    let half_w = world_width / 2.0;
    let half_h = world_height / 2.0;
    let y = WALL_HEIGHT + GHOST_HEIGHT + GRID_LINE_HEIGHT / 2.0;
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.5),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    for col in 0..=board.width {
        let x = col as f32 * CELL_SIZE - half_w;
        commands.spawn((
            GridLine,
            Mesh3d(meshes.add(Cuboid::new(GRID_LINE_WIDTH, GRID_LINE_HEIGHT, world_height))),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(x, y, 0.0),
        ));
    }

    for row in 0..=board.height {
        let z = row as f32 * CELL_SIZE - half_h;
        commands.spawn((
            GridLine,
            Mesh3d(meshes.add(Cuboid::new(world_width, GRID_LINE_HEIGHT, GRID_LINE_WIDTH))),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(0.0, y, z),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_tile_visual(
    entity: &mut EntityCommands,
    tile: EditorTile,
    position: (usize, usize),
    world_width: f32,
    world_height: f32,
    tile_meshes: &EditorMeshAssets,
    wall_graphics: &WallGraphicsAssets,
    agent_graphics: &AgentGraphicsAssets,
    goblet_graphics: &GobletGraphicsAssets,
) {
    match tile {
        EditorTile::Empty => {
            entity.remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>)>();
        }
        EditorTile::Wall => {
            let translation =
                cell_to_world(position, CELL_SIZE, world_width, world_height, WALL_HEIGHT / 2.0);
            entity.insert((
                Mesh3d(tile_meshes.wall.clone()),
                MeshMaterial3d(wall_graphics.material.clone()),
                Transform::from_translation(translation),
            ));
        }
        EditorTile::Agent => {
            let translation = cell_to_world(
                position,
                CELL_SIZE,
                world_width,
                world_height,
                AGENT_HEIGHT / 2.0,
            );
            entity.insert((
                Mesh3d(tile_meshes.agent.clone()),
                MeshMaterial3d(agent_graphics.material.clone()),
                Transform::from_translation(translation),
            ));
        }
        EditorTile::Ghost => {
            let translation = cell_to_world(
                position,
                CELL_SIZE,
                world_width,
                world_height,
                WALL_HEIGHT + GHOST_HEIGHT / 2.0,
            );
            entity.insert((
                Mesh3d(tile_meshes.ghost.clone()),
                MeshMaterial3d(agent_graphics.ghost_material.clone()),
                Transform::from_translation(translation),
            ));
        }
        EditorTile::Goblet(reward) => {
            let translation = cell_to_world(
                position,
                CELL_SIZE,
                world_width,
                world_height,
                GOBLET_HEIGHT / 2.0,
            );
            let material = if reward > 0 {
                goblet_graphics.material.clone()
            } else {
                goblet_graphics.false_material.clone()
            };
            entity.insert((
                Mesh3d(tile_meshes.goblet.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(translation),
            ));
        }
    }
}

pub(crate) fn setup_tiles(
    mut commands: Commands,
    board: Res<EditorBoard>,
    tile_meshes: Res<EditorMeshAssets>,
    wall_graphics: Res<WallGraphicsAssets>,
    agent_graphics: Res<AgentGraphicsAssets>,
    goblet_graphics: Res<GobletGraphicsAssets>,
) {
    let (world_width, world_height) = world_dimensions(board.width, board.height, CELL_SIZE);

    for (row, cols) in board.tiles.iter().enumerate() {
        for (col, &tile) in cols.iter().enumerate() {
            let mut entity = commands.spawn(EditorTileEntity { row, col });
            apply_tile_visual(
                &mut entity,
                tile,
                (col, row),
                world_width,
                world_height,
                &tile_meshes,
                &wall_graphics,
                &agent_graphics,
                &goblet_graphics,
            );
        }
    }
}

pub(crate) fn sync_tiles(
    mut commands: Commands,
    board: Res<EditorBoard>,
    tile_meshes: Res<EditorMeshAssets>,
    wall_graphics: Res<WallGraphicsAssets>,
    agent_graphics: Res<AgentGraphicsAssets>,
    goblet_graphics: Res<GobletGraphicsAssets>,
    q: Query<(Entity, &EditorTileEntity)>,
) {
    let (world_width, world_height) = world_dimensions(board.width, board.height, CELL_SIZE);

    for (entity, EditorTileEntity { row, col }) in &q {
        let tile = board.tiles[*row][*col];
        let mut entity_commands = commands.entity(entity);
        apply_tile_visual(
            &mut entity_commands,
            tile,
            (*col, *row),
            world_width,
            world_height,
            &tile_meshes,
            &wall_graphics,
            &agent_graphics,
            &goblet_graphics,
        );
    }
}

/// Marker: a UI text label showing a goblet's `|reward|` (color already
/// conveys sign), tracking the tile's world position on screen. Editor-only
/// — the live game's `goblet/systems.rs::spawn_goblets` doesn't render this.
#[derive(Component)]
pub struct GobletRewardLabel {
    pub row: usize,
    pub col: usize,
}

/// Rebuilds the goblet-reward labels whenever the board changes. A full
/// despawn/respawn is simplest and correct — boards typically have only a
/// handful of goblets, so this isn't worth the bookkeeping a diff would need.
pub fn sync_goblet_labels(
    mut commands: Commands,
    board: Res<EditorBoard>,
    existing: Query<Entity, With<GobletRewardLabel>>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    for (row, cols) in board.tiles.iter().enumerate() {
        for (col, &tile) in cols.iter().enumerate() {
            if let EditorTile::Goblet(reward) = tile {
                commands.spawn((
                    GobletRewardLabel { row, col },
                    Text::new(reward.unsigned_abs().to_string()),
                    TextFont {
                        font_size: 16.0,
                        ..Default::default()
                    },
                    TextColor(Color::BLACK),
                    Node {
                        position_type: PositionType::Absolute,
                        ..Default::default()
                    },
                    Visibility::Visible,
                ));
            }
        }
    }
}

/// Projects each goblet label's world position to screen space every frame,
/// so it tracks correctly as the camera pans/zooms. The camera looks
/// straight down (no tilt), so the Y height used here doesn't affect the
/// projected X/Z screen position — it's just for readability of the code.
pub fn update_goblet_label_positions(
    board: Res<EditorBoard>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut labels: Query<(&GobletRewardLabel, &mut Node, &mut Visibility)>,
) {
    let Ok((camera, cam_transform)) = camera_q.single() else {
        return;
    };
    let (world_width, world_height) = world_dimensions(board.width, board.height, CELL_SIZE);

    for (label, mut node, mut visibility) in &mut labels {
        let world_pos = cell_to_world(
            (label.col, label.row),
            CELL_SIZE,
            world_width,
            world_height,
            1.6, // just above the goblet mesh (Cylinder height 3.0, centered at y=0)
        );

        match camera.world_to_viewport(cam_transform, world_pos) {
            Ok(screen_pos) => {
                *visibility = Visibility::Visible;
                // Rough centering for 1-3 digit numbers at font_size 16.
                node.left = Val::Px(screen_pos.x - 6.0);
                node.top = Val::Px(screen_pos.y - 8.0);
            }
            Err(_) => {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

/// The grid is visible while manually enabled, and auto-hides only once a
/// cell occupies fewer than [`MIN_GRID_CELL_SIZE_PX`] logical screen pixels.
pub fn sync_grid_visibility(
    state: Res<EditorState>,
    camera_q: Query<&Projection, With<Camera3d>>,
    mut grid_lines: Query<&mut Visibility, With<GridLine>>,
) {
    let Ok(Projection::Orthographic(ortho)) = camera_q.single() else {
        return;
    };
    let cell_screen_size = CELL_SIZE / ortho.scale.abs();
    let visible = state.show_grid && cell_screen_size >= MIN_GRID_CELL_SIZE_PX;
    let visibility = if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut grid_line in &mut grid_lines {
        *grid_line = visibility;
    }
}
