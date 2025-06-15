use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_color::palettes::css;
use bevy_math::bounding::BoundingVolume;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_render::primitives::Aabb;
use rendering::marching_cubes::display_stage::VoxeledRendered;
use rendering::marching_cubes::{MarchingCubesPlugin, VoxelVolume};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoNoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            PanOrbitCameraPlugin,
            MarchingCubesPlugin,
        ))
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    voxel_volume: Res<VoxelVolume>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PanOrbitCamera::default(),
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, -4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Voxel volume
    commands.spawn((
        Visibility::default(),
        Transform::default(),
        Aabb {
            center: voxel_volume.aabb.center(),
            half_extents: voxel_volume.aabb.half_size(),
        },
        VoxeledRendered,
    ));

    // Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(5.0)))),
        MeshMaterial3d(materials.add(Color::Srgba(css::MAGENTA))),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));

    // Cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_length(1.0))),
        MeshMaterial3d(materials.add(Color::Srgba(css::ORANGE_RED))),
        Transform::from_xyz(2.0, 0.0, 2.0),
    ));

    // Light
    // commands.spawn((
    //     PointLight {
    //         shadows_enabled: true,
    //         intensity: 10_000_000.,
    //         range: 10.0,
    //         ..default()
    //     },
    //     Transform::from_xyz(0.0, 8.0, 0.0),
    // ));
}
