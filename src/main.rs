use bevy::prelude::*;

use board_plugin::BoardPlugin;

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
    app.add_plugins(BoardPlugin);
    // Debug hierarchy inspector
    #[cfg(feature = "debug")]
    {
        use bevy_inspector_egui::bevy_egui::EguiPlugin;
        use bevy_inspector_egui::quick::WorldInspectorPlugin;

        use board_plugin::components::Coordinates;

        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()));
        app.register_type::<Coordinates>();
    }
    // Startup system (cameras)
    app.add_systems(Startup, camera_setup);
    // Run the app
    app.run();
}

fn camera_setup(mut commands: Commands) {
    // 2D orthographic camera
    commands.spawn(Camera2d);
}
