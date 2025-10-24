use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    agent::Action,
    core::{GGConfig, Policy},
    game_state::{EntityType, GameState, Goblet, HoverBox, HoverBoxText, HoverCell},
    scene::GroundPlane,
};

pub fn cell_to_world(
    position: (usize, usize),
    cell_size: f32,
    world_width: f32,
    world_height: f32,
) -> Vec3 {
    Vec3::new(
        position.0 as f32 * cell_size + cell_size / 2.0 - world_width / 2.0,
        0.0,
        position.1 as f32 * cell_size + cell_size / 2.0 - world_height / 2.0,
    )
}

pub fn cursor_to_grid_cell(
    windows: Query<&Window, With<PrimaryWindow>>,
    cams: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    plane_q: Query<&GlobalTransform, With<GroundPlane>>,
    config: Res<GGConfig>,
    mut hover: ResMut<HoverCell>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = cams.single() else {
        return;
    };
    let Ok(plane_gt) = plane_q.single() else {
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        *hover = HoverCell::default();
        return;
    };

    // World ray
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor) else {
        return;
    };
    let ro = ray.origin;
    let rd = ray.direction;

    // Infinite plane at the ground's origin with its world-space normal
    let plane_point = plane_gt.translation();
    let plane_normal = plane_gt.up().into();

    let denom = rd.dot(plane_normal);
    if denom.abs() < 1e-6 {
        *hover = HoverCell::default();
        return;
    }
    let t = (plane_point - ro).dot(plane_normal) / denom;
    if t < 0.0 {
        *hover = HoverCell::default();
        return;
    }

    let hit = ro + t * rd;

    // Convert the world hit to the ground's LOCAL (mesh) space.
    // In mesh space, X/Z are ~[-0.5, 0.5] because the mesh is a unit cuboid.
    let local = plane_gt.affine().inverse().transform_point3(hit); // or compute_matrix().inverse() 👍 :contentReference[oaicite:0]{index=0}

    // Grid geometry
    let world_w = config.world_generation.world_width;
    let world_h = config.world_generation.world_height;
    let cell = config.world_generation.cell_size;

    // Number of cells (prefer floor to avoid off-by-one at the far edge)
    let grid_w = (world_w / cell).floor() as u32;
    let grid_h = (world_h / cell).floor() as u32;

    // Map mesh-local [-0.5,0.5] → [0, grid_w/grid_h)
    let u_cells = (local.x + 0.5) * grid_w as f32; // columns
    let v_cells = (local.z + 0.5) * grid_h as f32; // rows

    let col = u_cells.floor() as i32;
    let row = v_cells.floor() as i32;

    if col < 0 || row < 0 || col as u32 >= grid_w || row as u32 >= grid_h {
        *hover = HoverCell::default();
        return;
    }

    hover.cell = Some(UVec2::new(col as u32, row as u32));
    hover.world_hit = Some(hit);
}

pub fn setup_hover_box(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BorderRadius::all(Val::Px(4.0)),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            GlobalZIndex(10),
            HoverBox,
            Name::new("HoverBox"),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextLayout::new_with_justify(Justify::Left),
                HoverBoxText,
            ));
        });
}

pub fn update_hover_box(
    windows: Query<&Window, With<PrimaryWindow>>,
    policy: Res<Policy>,
    game_state: Res<GameState>,
    mut q_box: Query<(Entity, &mut Node, &mut BackgroundColor), With<HoverBox>>,
    mut q_text: Query<(&mut Text, &ChildOf), With<HoverBoxText>>,
    hover: Res<HoverCell>,
) {
    let Ok(window) = windows.single() else {
        info!("No window for tooltip");
        return;
    };

    let Ok((box_entity, mut node, mut _bg)) = q_box.single_mut() else {
        info!("No tooltip box");
        return;
    };

    let mut text = {
        let mut found: Option<Mut<Text>> = None;
        for (t, parent) in &mut q_text {
            if parent.parent() == box_entity {
                found = Some(t);
                break;
            }
        }
        match found {
            Some(t) => t,
            None => {
                info!("No tooltip text under the hover box");
                node.display = Display::None;
                return;
            }
        }
    };

    let (Some(cell), Some(_world)) = (hover.cell, hover.world_hit) else {
        node.display = Display::None;
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        node.display = Display::None;
        return;
    };

    const OFFSET: Vec2 = Vec2::new(0.0, 0.0);
    const BOX_W: f32 = 320.0;
    const BOX_H: f32 = 120.0;

    let mut x = cursor.x + OFFSET.x;
    let mut y = cursor.y + OFFSET.y;

    if x + BOX_W > window.width() - 4.0 {
        x = (window.width() - 4.0) - BOX_W;
    }
    if y + BOX_H > window.height() - 4.0 {
        y = (window.height() - 4.0) - BOX_H;
    }

    node.left = Val::Px(x.max(4.0));
    node.top = Val::Px(y.max(4.0));
    node.display = Display::Grid;

    text.0 = format!(
        "Cell:         ({},{})\n\
        Reward: {}\n\
        Policy Action: {}",
        cell.x,
        cell.y,
        if let Some(&Goblet { reward, .. }) = game_state
            .board
            .goblets
            .iter()
            .find(|g| g.position.0 == cell.x as usize && g.position.1 == cell.y as usize)
        {
            reward
        } else {
            0
        },
        policy.0[(cell.y * (game_state.board.width as u32) + cell.x) as usize]
    );
}

pub fn thicker_gizmos(mut store: ResMut<GizmoConfigStore>) {
    let (cfg, _group) = store.config_mut::<DefaultGizmoConfigGroup>();
    cfg.line.width = 6.0; // thicker lines (default is 2.0)
}

pub fn visualize_policy(
    mut gizmos: Gizmos,
    policy: Res<Policy>,
    game_state: Res<GameState>,
    config: Res<GGConfig>,
) {
    let cell_size = config.world_generation.cell_size;
    let world_width = config.world_generation.world_width;
    let world_height = config.world_generation.world_height;

    for (i, action) in policy.0.iter().enumerate() {
        let col = (i as u32) % (world_width / cell_size) as u32;
        let row = (i as u32) / (world_width / cell_size) as u32;

        if game_state
            .board
            .wall_positions
            .contains(&(col as usize, row as usize))
        {
            continue;
        }

        match game_state.board.get(&(col as usize, row as usize)) {
            EntityType::Wall() | EntityType::Goblet(_) => continue,
            _ => {}
        }

        let mut center = cell_to_world(
            (col as usize, row as usize),
            cell_size,
            world_width,
            world_height,
        );
        center.y = 5.0;

        let dir = match action {
            Action::Up => Vec3::new(0.0, 0.0, -1.0),
            Action::Down => Vec3::new(0.0, 0.0, 1.0),
            Action::Left => Vec3::new(-1.0, 0.0, 0.0),
            Action::Right => Vec3::new(1.0, 0.0, 0.0),
        };

        let arrow_start = center - dir * 0.9 * cell_size / 2.0;
        let arrow_end = center + dir * 0.9 * cell_size / 2.0;
        let arrow_length = (arrow_end - arrow_start).length();

        gizmos
            .arrow(arrow_start, arrow_end, Color::BLACK)
            .with_tip_length(arrow_length * 0.2);
    }
}
