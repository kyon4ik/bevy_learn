use std::f32::consts::PI;

use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::css::SILVER;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use rendering::edge_detection::{EdgeDetection, EdgeDetectionPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EdgeDetectionPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate, update_settings))
        .run();
}

#[derive(Component)]
struct Rotates;

const SHAPES_X_EXTENT: f32 = 14.0;
const EXTRUSION_X_EXTENT: f32 = 16.0;
const Z_EXTENT: f32 = 5.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

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

    let extrusions = [
        meshes.add(Extrusion::new(Rectangle::default(), 1.)),
        meshes.add(Extrusion::new(Capsule2d::default(), 1.)),
        meshes.add(Extrusion::new(Annulus::default(), 1.)),
        meshes.add(Extrusion::new(Circle::default(), 1.)),
        meshes.add(Extrusion::new(Ellipse::default(), 1.)),
        meshes.add(Extrusion::new(RegularPolygon::default(), 1.)),
        meshes.add(Extrusion::new(Triangle2d::default(), 1.)),
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

    let num_extrusions = extrusions.len();

    for (i, shape) in extrusions.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(debug_material.clone()),
            Transform::from_xyz(
                -EXTRUSION_X_EXTENT / 2.
                    + i as f32 / (num_extrusions - 1) as f32 * EXTRUSION_X_EXTENT,
                2.0,
                -Z_EXTENT / 2.,
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Rotates,
        ));
    }

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));

    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        Msaa::Off,
        EdgeDetection::default(),
    ));

    // UI
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

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

#[derive(Component)]
struct SettingsText;

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
