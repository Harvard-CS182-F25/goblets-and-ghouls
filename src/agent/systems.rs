use bevy::prelude::*;

use crate::agent::{Agent, GhostAgent, PlayerActionMessage};
use crate::core::{GGConfig, Policy, PolicyTimer};
use crate::game_state::GameState;
use crate::scene::WALL_HEIGHT;

use super::components::{AgentBundle, GhostAgentBundle};
use super::visual::AgentGraphicsAssets;

pub fn cell_to_world(
    position: (usize, usize),
    cell_size: f32,
    world_width: f32,
    world_height: f32,
    is_ghost: bool,
) -> Vec3 {
    Vec3::new(
        position.0 as f32 * cell_size + cell_size / 2.0 - world_width / 2.0,
        if is_ghost { WALL_HEIGHT + 1.0 } else { 0.0 },
        position.1 as f32 * cell_size + cell_size / 2.0 - world_height / 2.0,
    )
}

pub fn spawn_agents(
    mut commands: Commands,
    meshes: Option<ResMut<Assets<Mesh>>>,
    graphics: Option<Res<AgentGraphicsAssets>>,
    state: Res<GameState>,
    config: Res<GGConfig>,
) {
    let agent_world_position = cell_to_world(
        state.board.agent_position,
        config.world_generation.cell_size,
        config.world_generation.world_width,
        config.world_generation.world_height,
        false,
    );

    info!("Spawning agent at position: {:?}", agent_world_position);

    let entity = commands
        .spawn(AgentBundle::new(&config.agent.name, agent_world_position))
        .id();

    let ghost_entity = if let Some(ghost_position) = state.board.ghost_position {
        let ghost_world_position = cell_to_world(
            ghost_position,
            config.world_generation.cell_size,
            config.world_generation.world_width,
            config.world_generation.world_height,
            true,
        );
        info!(
            "Spawning ghost agent at position: {:?}",
            ghost_world_position
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
        && let Some(mut meshes) = meshes
    {
        let mesh = meshes.add(Cuboid::new(
            config.world_generation.cell_size,
            config.world_generation.cell_size,
            config.world_generation.cell_size,
        ));

        commands.entity(entity).insert((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(graphics.material.clone()),
        ));

        if let Some(ghost_entity) = ghost_entity {
            commands.entity(ghost_entity).insert((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(graphics.ghost_material.clone()),
            ));
        }
    }
}

pub fn evaluate_policy(
    mut message_writer: MessageWriter<PlayerActionMessage>,
    mut timer: ResMut<PolicyTimer>,
    time: Res<Time>,
    game_state: Res<GameState>,
    policy: Res<Policy>,
) {
    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }

    let agent_position = game_state.board.agent_position;
    let action_index = agent_position.0 + agent_position.1 * game_state.board.width;
    let action = policy
        .0
        .get(action_index)
        .cloned()
        .expect("Policy action out of bounds");

    message_writer.write(PlayerActionMessage { action });
}

#[allow(clippy::type_complexity)]
pub fn step(
    mut message_reader: MessageReader<PlayerActionMessage>,
    mut query: Query<(&mut Transform, Option<&Agent>, Option<&GhostAgent>)>,
    mut game_state: ResMut<GameState>,
) {
    for &PlayerActionMessage { action } in message_reader.read() {
        let state = game_state.step(action);

        for (mut transform, is_agent, is_ghost) in query.iter_mut() {
            transform.translation = cell_to_world(
                if is_agent.is_some() {
                    state.board.agent_position
                } else if is_ghost.is_some() {
                    state
                        .board
                        .ghost_position
                        .expect("Ghost position should exist")
                } else {
                    continue;
                },
                game_state.config.world_generation.cell_size,
                game_state.config.world_generation.world_width,
                game_state.config.world_generation.world_height,
                is_ghost.is_some(),
            );
        }

        *game_state = state;
    }
}
