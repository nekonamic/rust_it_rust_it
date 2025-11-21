use bevy::{
    asset::UnapprovedPathMode,
    prelude::*,
    window::{PresentMode, WindowResolution},
};

mod resources;
mod screens;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Rust*it!!Rust*it!!".into(),
                        name: Some("bevy.app".into()),
                        resolution: WindowResolution::new(1920, 1080)
                            .with_scale_factor_override(1.),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..default()
                }),
            screens::plugin,
        ))
        .add_systems(Startup, spawn_camera)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Msaa::Sample4));
}
