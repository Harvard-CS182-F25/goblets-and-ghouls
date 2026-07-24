use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
            FrameTimeDiagnosticsPlugin::default(),
        ));
    }
}
