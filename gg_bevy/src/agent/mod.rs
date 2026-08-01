mod components;
mod systems;
mod visual;

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use gg_core::GhostPolicy;

pub use components::*;
pub use visual::*;

use crate::core::StartupSets;
use crate::resources::{ConfigResource, RestartedThisFrame, SimulationPaused};

fn ghost_is_teleop(config: Res<ConfigResource>) -> bool {
    config.0.agent.ghost_policy == Some(GhostPolicy::Teleop)
}

pub struct AgentPlugin;
impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<components::PlayerActionMessage>();
        app.insert_resource(SimulationPaused::default());
        app.insert_resource(RestartedThisFrame::default());
        app.add_systems(First, systems::clear_restart_flag);
        app.add_systems(PreStartup, spawn_agent_assets);
        app.add_systems(
            Startup,
            (
                systems::spawn_agents.in_set(StartupSets::Agents),
                systems::spawn_pause_indicator,
            ),
        );
        app.add_systems(
            Update,
            (
                systems::restart_when_done.run_if(input_just_pressed(KeyCode::Space)),
                // render_delay_secs / the policy timer only drives stepping
                // when the ghost isn't teleop-controlled; in Teleop mode the
                // human's keypress is what advances the simulation instead.
                systems::evaluate_policy.run_if(|config: Res<ConfigResource>| !ghost_is_teleop(config)),
                systems::evaluate_policy_teleop.run_if(ghost_is_teleop),
                systems::step,
                // Space pauses/plays the auto-driven simulation — but only
                // when Teleop isn't using Space for "stay in place" instead.
                systems::toggle_pause.run_if(
                    input_just_pressed(KeyCode::Space)
                        .and(|config: Res<ConfigResource>| !ghost_is_teleop(config)),
                ),
                systems::update_pause_indicator,
            ),
        );
    }
}

fn spawn_agent_assets(mut commands: Commands, config: Res<ConfigResource>) {
    if config.0.headless {
        return;
    }

    commands.init_resource::<visual::AgentGraphicsAssets>();
}
