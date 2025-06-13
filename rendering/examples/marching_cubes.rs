use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use rendering::marching_cubes::{MarchingCubesPlugin, VoxelVolume};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin, MarchingCubesPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        PanOrbitCamera::default(),
        Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
    ));

    commands.insert_resource(VoxelVolume::default());
}
