use bevy::prelude::*;

#[cfg(feature = "debug")]
use bevy_inspector_egui::bevy_egui::EguiPlugin;
#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    let mut app = App::new();
    // Bevy default plugins with window setup
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Mine Sweeper!".to_string(),
            resolution: (700., 800.).into(),
            ..Default::default()
        }),
        ..Default::default()
    }));
    #[cfg(feature = "debug")]
    // Debug hierarchy inspector
    app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()));
    // Startup system (cameras)
    app.add_systems(Startup, camera_setup);
    // Run the app
    app.run();
}

fn camera_setup(mut commands: Commands) {
    // 2D orthographic camera
    commands.spawn(Camera2d);
}
