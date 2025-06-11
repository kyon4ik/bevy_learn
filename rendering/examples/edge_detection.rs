use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_core_pipeline::motion_blur::MotionBlur;
use bevy_core_pipeline::smaa::Smaa;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use rendering::edge_detection::{EdgeDetection, EdgeDetectionPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin, EdgeDetectionPlugin))
        .add_systems(
            Startup,
            (spawn_scene, spawn_lights_and_camera, spawn_ui).chain(),
        )
        .add_systems(Update, (rotate, update_settings))
        .run();
}

#[derive(Component)]
struct Rotates;

const SHAPES_X_EXTENT: f32 = 14.0;
const Z_EXTENT: f32 = 5.0;

fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(Color::WHITE);

    let shapes = [
        meshes.add(Cuboid::default()),
        meshes.add(Tetrahedron::default()),
        meshes.add(Capsule3d::default()),
        meshes.add(Torus::default()),
        meshes.add(Cylinder::default()),
        meshes.add(Cone::default()),
        meshes.add(ConicalFrustum::default()),
        meshes.add(Sphere::default().mesh().ico(5).unwrap()),
        meshes.add(Sphere::default().mesh().uv(32, 18)),
    ];

    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(debug_material.clone()),
            Transform::from_xyz(
                -SHAPES_X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * SHAPES_X_EXTENT,
                2.0,
                Z_EXTENT / 2.,
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Rotates,
        ));
    }

    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(debug_material),
    ));
}

fn spawn_lights_and_camera(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::WHITE));

    commands.spawn((
        PanOrbitCamera::default(),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        Msaa::Off,
        EdgeDetection::default(),
        Camera {
            hdr: true,
            ..Default::default()
        },
        Smaa::default(),
        MotionBlur {
            shutter_angle: 0.25,
            samples: 2,
        },
    ));

    commands.spawn((
        PointLight {
            intensity: 10_000_000.,
            range: 100.0,
            ..Default::default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));
}

#[derive(Component)]
struct SettingsText;

fn spawn_ui(mut commands: Commands) {
    commands.spawn((
        Text::default(),
        TextColor(Color::srgb(1.0, 0.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        SettingsText,
    ));
}

fn rotate(mut query: Query<&mut Transform, With<Rotates>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() / 2.);
    }
}

fn axis_control(keyboard: &ButtonInput<KeyCode>, left: KeyCode, right: KeyCode, value: f32) -> f32 {
    let mut res = 0.0;
    if keyboard.pressed(left) {
        res -= value;
    }
    if keyboard.pressed(right) {
        res += value;
    }
    res
}

fn update_settings(
    camera_query: Single<(Entity, Option<&mut EdgeDetection>), With<Camera>>,
    mut text: Single<&mut Text, With<SettingsText>>,
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (camera_entity, edge_detection) = camera_query.into_inner();

    if let Some(mut edge_detection) = edge_detection {
        text.0 = "Edge Detection (Toggle: Space)\n".to_string();
        text.push_str(&format!(
            "(A/Q) Low Depth Threshold: {}\n",
            edge_detection.depth_threshold.x
        ));
        text.push_str(&format!(
            "(S/W) High Depth Threshold: {}\n",
            edge_detection.depth_threshold.y
        ));
        text.push_str(&format!(
            "(D/E) Low Normal Threshold: {}\n",
            edge_detection.normal_threshold.x
        ));
        text.push_str(&format!(
            "(F/R) High Normal Threshold: {}\n",
            edge_detection.normal_threshold.y
        ));
        text.push_str(&format!("(G/T) Width: {}\n", edge_detection.width));
        text.push_str(&format!(
            "(H/Y) Final Threshold: {}\n",
            edge_detection.final_threshold
        ));

        if keyboard.just_pressed(KeyCode::Space) {
            commands.entity(camera_entity).remove::<EdgeDetection>();
        }

        let dt = time.delta_secs();
        use KeyCode::*;
        edge_detection.depth_threshold.x += axis_control(&keyboard, KeyA, KeyQ, dt / 5.0);
        edge_detection.depth_threshold.y += axis_control(&keyboard, KeyS, KeyW, dt / 5.0);
        edge_detection.normal_threshold.x += axis_control(&keyboard, KeyD, KeyE, dt / 2.0);
        edge_detection.normal_threshold.y += axis_control(&keyboard, KeyF, KeyR, dt / 2.0);
        edge_detection.width += axis_control(&keyboard, KeyG, KeyT, 2.0 * dt);
        edge_detection.final_threshold += axis_control(&keyboard, KeyH, KeyY, dt / 2.0);

        let elapsed = time.elapsed_secs();
        edge_detection.edge_color = Color::srgb(elapsed.sin(), elapsed.cos(), 0.3);
    } else {
        text.0 = "Edge Detection: Off (Toggle: Space)\n".to_string();

        if keyboard.just_pressed(KeyCode::Space) {
            commands
                .entity(camera_entity)
                .insert(EdgeDetection::default());
        }
    }
}
