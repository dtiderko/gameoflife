use bevy::prelude::*;
use bevy_egui::prelude::*;
use grid::*;

mod grid;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(Grid)
        .add_systems(Startup, setup_camera_system)
        .run()
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}
