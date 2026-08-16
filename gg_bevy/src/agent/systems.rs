use bevy::prelude::*;
use gg_core::Action;

use crate::coords::{cell_to_world, world_dimensions};
use crate::resources::{
    ConfigResource, GameStateResource, PolicyResource, PolicyTimer, RestartedThisFrame,
    SimulationPaused, SimulationSpeed,
};
use crate::scene::{AGENT_HEIGHT, GHOST_HEIGHT, RenderMeshAssets, WALL_HEIGHT};

use super::components::{
    Agent, AgentBundle, GhostAgent, GhostAgentBundle, GhostInput, PauseIndicatorBadge,
    PauseIndicatorText, PlayerActionMessage, SimulationSpeedText,
};
use super::visual::AgentGraphicsAssets;

pub fn spawn_agents(
    mut commands: Commands,
    meshes: Option<Res<RenderMeshAssets>>,
    graphics: Option<Res<AgentGraphicsAssets>>,
    state: Res<GameStateResource>,
    config: Res<ConfigResource>,
) {
    let cell = config.0.world_generation.cell_size;
    let (world_width, world_height) =
        world_dimensions(state.0.board.width, state.0.board.height, cell);

    let agent_world_position = cell_to_world(
        state.0.board.agent_position,
        cell,
        world_width,
        world_height,
        AGENT_HEIGHT / 2.0,
    );

    let entity = commands
        .spawn(AgentBundle::new(&config.0.agent.name, agent_world_position))
        .id();

    let ghost_entity = if let Some(ghost_position) = state.0.board.ghost_position {
        let ghost_world_position = cell_to_world(
            ghost_position,
            cell,
            world_width,
            world_height,
            WALL_HEIGHT + GHOST_HEIGHT / 2.0,
        );
        Some(
            commands
                .spawn(GhostAgentBundle::new("Ghost", ghost_world_position))
                .id(),
        )
    } else {
        None
    };

    if let Some(graphics) = graphics
        && let Some(meshes) = meshes
    {
        commands.entity(entity).insert((
            Mesh3d(meshes.agent.clone()),
            MeshMaterial3d(graphics.material.clone()),
        ));

        if let Some(ghost_entity) = ghost_entity {
            commands.entity(ghost_entity).insert((
                Mesh3d(meshes.ghost.clone()),
                MeshMaterial3d(graphics.ghost_material.clone()),
            ));
        }
    }
}

pub fn clear_restart_flag(mut restarted: ResMut<RestartedThisFrame>) {
    restarted.0 = false;
}

fn sync_actor_transforms(
    query: &mut Query<(&mut Transform, Option<&Agent>, Option<&GhostAgent>)>,
    state: &gg_core::GameState,
) {
    let cell = state.config.world_generation.cell_size;
    let (world_width, world_height) = world_dimensions(state.board.width, state.board.height, cell);

    for (mut transform, is_agent, is_ghost) in query.iter_mut() {
        let (position, y) = if is_agent.is_some() {
            (state.board.agent_position, AGENT_HEIGHT / 2.0)
        } else if is_ghost.is_some() {
            (
                state
                    .board
                    .ghost_position
                    .expect("Ghost position should exist"),
                WALL_HEIGHT + GHOST_HEIGHT / 2.0,
            )
        } else {
            continue;
        };

        transform.translation = cell_to_world(position, cell, world_width, world_height, y);
    }
}

pub fn restart_when_done(
    mut query: Query<(&mut Transform, Option<&Agent>, Option<&GhostAgent>)>,
    mut game_state: ResMut<GameStateResource>,
    mut timer: ResMut<PolicyTimer>,
    mut paused: ResMut<SimulationPaused>,
    mut restarted: ResMut<RestartedThisFrame>,
) {
    let current_seed = game_state.0.rng_seed;
    restart_done_with_seed(
        &mut query,
        &mut game_state,
        &mut timer,
        &mut paused,
        &mut restarted,
        Some(current_seed),
    );
}

pub fn restart_with_new_seed_when_done(
    mut query: Query<(&mut Transform, Option<&Agent>, Option<&GhostAgent>)>,
    mut game_state: ResMut<GameStateResource>,
    mut timer: ResMut<PolicyTimer>,
    mut paused: ResMut<SimulationPaused>,
    mut restarted: ResMut<RestartedThisFrame>,
) {
    restart_done_with_seed(
        &mut query,
        &mut game_state,
        &mut timer,
        &mut paused,
        &mut restarted,
        None,
    );
}

fn restart_done_with_seed(
    query: &mut Query<(&mut Transform, Option<&Agent>, Option<&GhostAgent>)>,
    game_state: &mut ResMut<GameStateResource>,
    timer: &mut ResMut<PolicyTimer>,
    paused: &mut ResMut<SimulationPaused>,
    restarted: &mut ResMut<RestartedThisFrame>,
    seed: Option<u64>,
) {
    if !game_state.0.done {
        return;
    }

    let (state, _) = game_state.0.reset(seed);
    game_state.0 = state;

    sync_actor_transforms(query, &game_state.0);

    timer.0.reset();
    paused.0 = false;
    restarted.0 = true;
}

pub fn evaluate_policy(
    mut message_writer: MessageWriter<PlayerActionMessage>,
    mut timer: ResMut<PolicyTimer>,
    time: Res<Time>,
    game_state: Res<GameStateResource>,
    policy: Res<PolicyResource>,
    config: Res<ConfigResource>,
    paused: Res<SimulationPaused>,
    restarted: Res<RestartedThisFrame>,
    speed: Res<SimulationSpeed>,
) {
    // Don't tick the timer at all while paused, so resuming continues from
    // wherever it left off rather than firing immediately.
    if paused.0 || restarted.0 {
        return;
    }

    timer.0.tick(time.delta().mul_f32(speed.multiplier()));
    if !timer.0.is_finished() {
        return;
    }

    let board = &game_state.0.board;
    let ghost_position = board.effective_ghost_position(config.0.agent.ghost_occlusion);
    let action = policy.0.get(board.agent_position, ghost_position);

    message_writer.write(PlayerActionMessage {
        action,
        ghost_input: GhostInput::Auto,
    });
}

/// Changes the automatic simulation speed with `[` and `]`.
pub fn adjust_simulation_speed(
    keys: Res<ButtonInput<KeyCode>>,
    mut speed: ResMut<SimulationSpeed>,
) {
    if keys.just_pressed(KeyCode::BracketLeft) {
        speed.slower();
    } else if keys.just_pressed(KeyCode::BracketRight) {
        speed.faster();
    }
}

/// Toggles `SimulationPaused`. Only registered when `GhostPolicy::Teleop`
/// isn't active (see `agent::mod::ghost_is_teleop`) — Space already means
/// "ghost stays in place" there, and pause/play doesn't apply to a
/// simulation that only ever advances on a keypress anyway.
pub fn toggle_pause(
    mut paused: ResMut<SimulationPaused>,
    game_state: Res<GameStateResource>,
    restarted: Res<RestartedThisFrame>,
) {
    if game_state.0.done || restarted.0 {
        return;
    }
    paused.0 = !paused.0;
}

/// Spawns the pause/play HUD badge in the upper-left corner (freed up once
/// the seed/reward info moved into the upper-right panel). Not spawned at
/// all when `GhostPolicy::Teleop` is active — pause/play isn't a meaningful
/// concept for a simulation that only ever advances on a keypress.
pub fn spawn_pause_indicator(
    mut commands: Commands,
    config: Res<ConfigResource>,
    speed: Res<SimulationSpeed>,
) {
    if config.0.agent.ghost_policy == Some(gg_core::GhostPolicy::Teleop) {
        return;
    }

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            PauseIndicatorBadge,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                PauseIndicatorText,
            ));
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(36.0),
                left: Val::Px(5.0),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.3, 0.3, 0.6, 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("SPEED {}x", speed.multiplier())),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                SimulationSpeedText,
            ));
        });
}

/// Recolors/relabels the pause badge whenever `SimulationPaused` changes —
/// green "PLAYING" or red "PAUSED". Plain ASCII text rather than a Unicode
/// ⏸/▶ glyph, which the default font may not render; color is the primary
/// signal either way.
pub fn update_pause_indicator(
    paused: Res<SimulationPaused>,
    mut badges: Query<&mut BackgroundColor, With<PauseIndicatorBadge>>,
    mut texts: Query<&mut Text, With<PauseIndicatorText>>,
) {
    if !paused.is_changed() {
        return;
    }

    let (color, label) = if paused.0 {
        (Color::srgba(0.75, 0.1, 0.1, 0.85), "PAUSED")
    } else {
        (Color::srgba(0.1, 0.75, 0.1, 0.85), "PLAYING")
    };

    for mut bg in &mut badges {
        *bg = BackgroundColor(color);
    }
    for mut text in &mut texts {
        text.0 = label.to_string();
    }
}

/// Keeps the speed badge in sync with speed changes.
pub fn update_speed_indicator(
    speed: Res<SimulationSpeed>,
    mut texts: Query<&mut Text, With<SimulationSpeedText>>,
) {
    if !speed.is_changed() {
        return;
    }

    for mut text in &mut texts {
        text.0 = format!("SPEED {}", format!("{}x", speed.multiplier()));
    }
}

/// `GhostPolicy::Teleop` counterpart to `evaluate_policy`: instead of firing
/// on a fixed timer (render_delay_secs is unused in this mode), the whole
/// simulation waits for the human controlling the ghost to press a
/// direction key (arrows or WASD) or Space ("stay in place") before
/// advancing one step.
pub fn evaluate_policy_teleop(
    mut message_writer: MessageWriter<PlayerActionMessage>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    game_state: Res<GameStateResource>,
    policy: Res<PolicyResource>,
    config: Res<ConfigResource>,
    restarted: Res<RestartedThisFrame>,
) {
    if game_state.0.done || restarted.0 {
        return;
    }

    let ghost_input = if keyboard_input.just_pressed(KeyCode::ArrowUp)
        || keyboard_input.just_pressed(KeyCode::KeyW)
    {
        GhostInput::TeleopMove(Action::Up)
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown)
        || keyboard_input.just_pressed(KeyCode::KeyS)
    {
        GhostInput::TeleopMove(Action::Down)
    } else if keyboard_input.just_pressed(KeyCode::ArrowLeft)
        || keyboard_input.just_pressed(KeyCode::KeyA)
    {
        GhostInput::TeleopMove(Action::Left)
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight)
        || keyboard_input.just_pressed(KeyCode::KeyD)
    {
        GhostInput::TeleopMove(Action::Right)
    } else if keyboard_input.just_pressed(KeyCode::Space) {
        GhostInput::TeleopStay
    } else {
        return;
    };

    let board = &game_state.0.board;
    let ghost_position = board.effective_ghost_position(config.0.agent.ghost_occlusion);
    let action = policy.0.get(board.agent_position, ghost_position);

    message_writer.write(PlayerActionMessage {
        action,
        ghost_input,
    });
}

#[allow(clippy::type_complexity)]
pub fn step(
    mut message_reader: MessageReader<PlayerActionMessage>,
    mut query: Query<(&mut Transform, Option<&Agent>, Option<&GhostAgent>)>,
    mut game_state: ResMut<GameStateResource>,
) {
    for &PlayerActionMessage {
        action,
        ghost_input,
    } in message_reader.read()
    {
        let state = match ghost_input {
            GhostInput::Auto => game_state.0.step(action),
            GhostInput::TeleopMove(direction) => game_state.0.step_teleop(action, Some(direction)),
            GhostInput::TeleopStay => game_state.0.step_teleop(action, None),
        };
        game_state.0 = state;
        sync_actor_transforms(&mut query, &game_state.0);
    }
}
