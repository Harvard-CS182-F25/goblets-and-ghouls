use bevy::prelude::*;
use gg_core::{GGConfig, GameState, Policy, ValueGrid};

use crate::resources::{ConfigResource, GameStateResource, HeatmapResource, PolicyResource, PolicyTimer};
use crate::{agent, camera, debug, game_state, goblet, scene};

#[derive(SystemSet, Debug, Clone, Hash, PartialEq, Eq)]
pub enum StartupSets {
    Agents,
    Goblets,
    Walls,
}

/// Builds the full game/render app around an already-constructed
/// `GameState` (procedural or loaded from a custom world file — the board
/// is never regenerated inside Bevy).
pub struct GGPlugin {
    pub initial_state: GameState,
    pub config: GGConfig,
    pub policy: Policy,
    pub value: Option<ValueGrid>,
}

impl Plugin for GGPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConfigResource(self.config.clone()));
        app.insert_resource(GameStateResource(self.initial_state.clone()));
        app.insert_resource(PolicyResource(self.policy.clone()));
        app.insert_resource(HeatmapResource(self.value.clone()));
        app.insert_resource(PolicyTimer(Timer::from_seconds(
            self.config.render_delay_secs,
            TimerMode::Repeating,
        )));

        if self.config.debug {
            app.add_plugins(debug::DebugPlugin);
        }

        app.add_plugins((
            agent::AgentPlugin,
            camera::CameraPlugin,
            goblet::GobletPlugin,
            scene::ScenePlugin,
            game_state::GameStatePlugin,
        ));

        app.configure_sets(
            Startup,
            (
                StartupSets::Walls,
                StartupSets::Goblets,
                StartupSets::Agents,
            )
                .chain(),
        );

        app.insert_resource(ClearColor(Color::srgb_u8(0, 136, 255)));
    }
}
