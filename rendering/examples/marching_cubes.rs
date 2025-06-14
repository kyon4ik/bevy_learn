use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use rendering::marching_cubes::MarchingCubesPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin, MarchingCubesPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        PanOrbitCamera::default(),
        Msaa::Off,
        Transform::from_xyz(0.0, 0.0, -4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
