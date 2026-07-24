use bevy::{prelude::*, window::PrimaryWindow};
use gg_core::{Action, EntityType, Goblet};

use crate::coords::{cell_to_world, raycast_to_grid_cell, world_dimensions};
use crate::resources::{ConfigResource, GameStateResource, HeatmapResource, PolicyResource};
use crate::scene::GroundPlane;

use super::components::{
    GameOverOverlay, HeatmapColorRange, HeatmapTile, HoverBox, HoverBoxText, HoverCell,
    VisibilityTile, VisualizePolicy, VisualizeValue, VisualizeVisibility,
};

pub fn cursor_to_grid_cell(
    windows: Query<&Window, With<PrimaryWindow>>,
    cams: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    plane_q: Query<&GlobalTransform, With<GroundPlane>>,
    state: Res<GameStateResource>,
    mut hover: ResMut<HoverCell>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = cams.single() else {
        return;
    };
    let Ok(plane_gt) = plane_q.single() else {
        return;
    };

    match raycast_to_grid_cell(
        window,
        camera,
        cam_transform,
        plane_gt,
        state.0.board.width as u32,
        state.0.board.height as u32,
    ) {
        Some((cell, hit)) => {
            hover.cell = Some(cell);
            hover.world_hit = Some(hit);
        }
        None => *hover = HoverCell::default(),
    }
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
    policy: Res<PolicyResource>,
    heatmap: Res<HeatmapResource>,
    game_state: Res<GameStateResource>,
    config: Res<ConfigResource>,
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

    let agent_position = (cell.x as usize, cell.y as usize);
    let ghost_position = game_state
        .0
        .board
        .effective_ghost_position_for(agent_position, config.0.agent.ghost_occlusion);
    let policy_action = policy.0.get(agent_position, ghost_position);

    let value_line = match &heatmap.0 {
        Some(grid) => format!("\nValue: {:.3}", grid.get(agent_position, ghost_position)),
        None => String::new(),
    };

    // Independent of ghost_occlusion (which only gates the ghost's own
    // visibility) — plain agent-to-cell line of sight, same predicate
    // `update_visibility_overlay` uses for the "F" overlay.
    let visible = game_state
        .0
        .board
        .has_line_of_sight(game_state.0.board.agent_position, agent_position);

    text.0 = format!(
        "Cell:         ({},{})\n\
        Reward: {}\n\
        Policy Action: {}{}\n\
        Visible: {}",
        cell.x,
        cell.y,
        if let Some(&Goblet { reward, .. }) = game_state.0.board.goblets.iter().find(|g| g
            .position
            .0
            == cell.x as usize
            && g.position.1 == cell.y as usize)
        {
            reward
        } else {
            0
        },
        policy_action,
        value_line,
        if visible { "Yes" } else { "No" }
    );
}

pub fn thicker_gizmos(mut store: ResMut<GizmoConfigStore>) {
    // `Gizmos::arrow` draws a 3D "4-fin" arrowhead (fins at local
    // (-1,±1,0)/(-1,0,±1)) meant to look right from any camera angle. Under
    // this game's strict top-down orthographic camera, half those fins have
    // a pure world-Y (depth) component that foreshortens to ~zero on
    // screen, piling up extra opaque quads right at the tip point. Combined
    // with `line.width` being a *screen-pixel* quantity (flat quads, no
    // rounded caps — see bevy_gizmos' lines.wgsl) that's large relative to
    // the short tip segments, wide values here make the tip read as a blob
    // instead of a chevron. Keep both widths modest and the tip long
    // relative to the shaft (see `with_tip_length` in `visualize_policy`)
    // to keep the arrowhead crisp.
    let (cfg, _group) = store.config_mut::<DefaultGizmoConfigGroup>();
    cfg.line.width = 3.0; // thicker than the 2.0 default, not so thick it blobs
}

pub fn toggle_policy_visualization(mut visualize_policy: ResMut<VisualizePolicy>) {
    visualize_policy.0 = !visualize_policy.0;
}

pub fn visualize_policy(
    mut gizmos: Gizmos,
    policy: Res<PolicyResource>,
    game_state: Res<GameStateResource>,
    config: Res<ConfigResource>,
    visualize_policy: Res<VisualizePolicy>,
) {
    if !visualize_policy.0 {
        return;
    }

    let cell_size = config.0.world_generation.cell_size;
    let board = &game_state.0.board;
    let (world_width, world_height) = world_dimensions(board.width, board.height, cell_size);

    for col in 0..board.width {
        for row in 0..board.height {
            if board.wall_positions.contains(&(col, row)) {
                continue;
            }

            match board.get(&(col, row)) {
                EntityType::Wall() | EntityType::Goblet(_) => continue,
                _ => {}
            }

            // Sweeping "if the agent were at this cell" per-cell (rather than
            // fixing one ghost coordinate for the whole sweep) is what makes
            // arrows fall back to the pi[x,y,x,y] diagonal exactly where the
            // ghost is absent/occluded from that hypothetical cell, while
            // still tracking the ghost's real position where it's visible.
            let agent_position = (col, row);
            let ghost_position =
                board.effective_ghost_position_for(agent_position, config.0.agent.ghost_occlusion);
            let action = policy.0.get(agent_position, ghost_position);

            let mut center =
                cell_to_world(agent_position, cell_size, world_width, world_height, 0.0);
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

            // Tip length is a larger fraction of the shaft here (0.3, vs
            // bevy_gizmos' own default of 0.1) so the chevron stays long
            // relative to `line.width` and reads as a clean "V" rather than
            // a blob — see the root-cause note on `thicker_gizmos`.
            gizmos
                .arrow(arrow_start, arrow_end, Color::BLACK)
                .with_tip_length(arrow_length * 0.3);
        }
    }
}

pub fn toggle_value_visualization(mut visualize_value: ResMut<VisualizeValue>) {
    visualize_value.0 = !visualize_value.0;
}

/// Spawns one flat, per-cell tile entity for every non-wall cell (goblet
/// cells included — their value is meaningful, unlike "what action to take"
/// for the policy arrows). Hidden by default; `update_heatmap` toggles
/// visibility and recolors them.
pub fn spawn_heatmap_tiles(
    mut commands: Commands,
    meshes: Option<ResMut<Assets<Mesh>>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
    config: Res<ConfigResource>,
    state: Res<GameStateResource>,
) {
    let (Some(mut meshes), Some(mut materials)) = (meshes, materials) else {
        return;
    };

    let cell_size = config.0.world_generation.cell_size;
    let board = &state.0.board;
    let (world_width, world_height) = world_dimensions(board.width, board.height, cell_size);

    let mesh = meshes.add(Cuboid::new(cell_size * 0.95, 0.05, cell_size * 0.95));

    for col in 0..board.width {
        for row in 0..board.height {
            if board.wall_positions.contains(&(col, row)) {
                continue;
            }

            let mut position = cell_to_world((col, row), cell_size, world_width, world_height, 0.0);
            // Just above the ground plane's top surface (a unit cuboid
            // scaled to (world_width, 1.0, world_height), so its top sits at
            // y=0.5), safely below the y=5.0 arrow overlay.
            position.y = 0.51;

            let material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            });

            commands.spawn((
                HeatmapTile((col, row)),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(position),
                Visibility::Hidden,
            ));
        }
    }
}

/// Recomputes the cached global min/max of the current `HeatmapResource`
/// (gated on `resource_changed::<HeatmapResource>` in the plugin) so tile
/// colors stay stable frame-to-frame rather than renormalizing constantly.
pub fn update_heatmap_color_range(
    heatmap: Res<HeatmapResource>,
    mut range: ResMut<HeatmapColorRange>,
) {
    range.0 = heatmap.0.as_ref().map(|grid| {
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for ax in 0..grid.width {
            for ay in 0..grid.height {
                for gx in 0..grid.width {
                    for gy in 0..grid.height {
                        let v = grid.get((ax, ay), (gx, gy));
                        min = min.min(v);
                        max = max.max(v);
                    }
                }
            }
        }
        (min, max)
    });
}

/// Diverging colormap centered at 0: white at zero, green toward the
/// magnitude-normalized positive extreme, red toward the negative one.
fn value_to_color(value: f32, min: f32, max: f32) -> Color {
    let magnitude_scale = min.abs().max(max.abs()).max(1e-6);
    let t = (value / magnitude_scale).clamp(-1.0, 1.0);
    if t >= 0.0 {
        Color::srgb(1.0 - t, 1.0, 1.0 - t)
    } else {
        let t = -t;
        Color::srgb(1.0, 1.0 - t, 1.0 - t)
    }
}

pub fn update_heatmap(
    visualize_value: Res<VisualizeValue>,
    heatmap: Res<HeatmapResource>,
    range: Res<HeatmapColorRange>,
    game_state: Res<GameStateResource>,
    config: Res<ConfigResource>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(
        &HeatmapTile,
        &MeshMaterial3d<StandardMaterial>,
        &mut Visibility,
    )>,
) {
    let Some(grid) = &heatmap.0 else {
        for (_, _, mut visibility) in &mut query {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    if !visualize_value.0 {
        for (_, _, mut visibility) in &mut query {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let Some((min, max)) = range.0 else { return };
    let board = &game_state.0.board;

    for (tile, material_handle, mut visibility) in &mut query {
        *visibility = Visibility::Visible;
        let effective_ghost =
            board.effective_ghost_position_for(tile.0, config.0.agent.ghost_occlusion);
        let value = grid.get(tile.0, effective_ghost);
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color = value_to_color(value, min, max);
        }
    }
}

pub fn toggle_visibility_overlay(mut visualize_visibility: ResMut<VisualizeVisibility>) {
    visualize_visibility.0 = !visualize_visibility.0;
}

/// Spawns one flat, per-cell tile entity for every non-wall cell — the same
/// footprint as the heatmap tiles, but layered just above them (y=0.6, still
/// safely below the y=5.0 arrow overlay) since this overlay darkens
/// whatever's beneath it. Hidden by default; `update_visibility_overlay`
/// toggles and darkens tiles the agent currently can't see.
pub fn spawn_visibility_tiles(
    mut commands: Commands,
    meshes: Option<ResMut<Assets<Mesh>>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
    config: Res<ConfigResource>,
    state: Res<GameStateResource>,
) {
    let (Some(mut meshes), Some(mut materials)) = (meshes, materials) else {
        return;
    };

    let cell_size = config.0.world_generation.cell_size;
    let board = &state.0.board;
    let (world_width, world_height) = world_dimensions(board.width, board.height, cell_size);

    let mesh = meshes.add(Cuboid::new(cell_size * 0.95, 0.05, cell_size * 0.95));

    for col in 0..board.width {
        for row in 0..board.height {
            if board.wall_positions.contains(&(col, row)) {
                continue;
            }

            let mut position = cell_to_world((col, row), cell_size, world_width, world_height, 0.0);
            position.y = 0.6;

            let material = materials.add(StandardMaterial {
                base_color: Color::srgba(0.0, 0.0, 0.0, 0.55),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

            commands.spawn((
                VisibilityTile((col, row)),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(position),
                Visibility::Hidden,
            ));
        }
    }
}

/// Darkens every tile not currently visible from the agent's position (per
/// `Board::has_line_of_sight`, independent of `ghost_occlusion` — this is
/// about the agent's own sightlines, not specifically about the ghost).
pub fn update_visibility_overlay(
    visualize_visibility: Res<VisualizeVisibility>,
    game_state: Res<GameStateResource>,
    mut query: Query<(&VisibilityTile, &mut Visibility)>,
) {
    if !visualize_visibility.0 {
        for (_, mut visibility) in &mut query {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let board = &game_state.0.board;
    let agent_position = board.agent_position;

    for (tile, mut visibility) in &mut query {
        *visibility = if board.has_line_of_sight(agent_position, tile.0) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

/// Full-screen semi-transparent banner announcing why the episode ended —
/// mirrors `segmentation`'s `show_winner_overlay` (same layout: centered
/// 72px text over a 65%-black full-screen `Node`). Spawned once `done`
/// becomes true; torn down again if the episode ever resets (nothing in
/// `gg_bevy` currently triggers a reset, but this keeps the system correct
/// if one is added later).
pub fn show_game_over_overlay(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    existing: Query<Entity, With<GameOverOverlay>>,
) {
    if !game_state.0.done {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
        return;
    }

    if !existing.is_empty() {
        return;
    }

    let (message, text_color) = if game_state.0.reward == i32::MIN {
        ("Caught", Color::srgb(0.85, 0.15, 0.15))
    } else if game_state.0.reward > 0 {
        ("Escaped with Gold", Color::srgb(0.15, 0.75, 0.15))
    } else {
        ("Escaped with Fool's Gold", Color::srgb(0.85, 0.15, 0.15))
    };

    let text = commands
        .spawn((
            Text::new(message),
            TextFont {
                font_size: 72.0,
                ..default()
            },
            TextColor(text_color),
        ))
        .id();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            GlobalZIndex(10),
            GameOverOverlay,
        ))
        .add_children(&[text]);
}
