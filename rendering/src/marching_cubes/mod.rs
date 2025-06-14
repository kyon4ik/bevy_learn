use bevy_app::{App, Plugin};
use bevy_asset::{Assets, Handle, RenderAssetUsages};
use bevy_ecs::resource::Resource;
use bevy_ecs::world::FromWorld;
use bevy_math::Vec3;
use bevy_render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy_render::render_resource::BufferUsages;
use bevy_render::storage::ShaderStorageBuffer;

use bytemuck::{Pod, Zeroable};
pub use compute_stage::VoxelVolume;

pub mod compute_stage;
pub mod display_stage;

pub struct MarchingCubesPlugin;

impl Plugin for MarchingCubesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(compute_stage::MarchingCubesComputePlugin);

        app.init_resource::<MarchingCubesBuffers>();
        app.add_plugins(ExtractResourcePlugin::<MarchingCubesBuffers>::default());

        app.add_plugins(display_stage::MarchingCubesDisplayPlugin);
    }
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: Vec3,
    _pad0: f32,
}

#[derive(Resource, Clone, ExtractResource)]
pub struct MarchingCubesBuffers {
    vertices: Handle<ShaderStorageBuffer>,
}

impl FromWorld for MarchingCubesBuffers {
    fn from_world(world: &mut bevy_ecs::world::World) -> Self {
        let voxel_volume = world.resource::<VoxelVolume>();
        let buffer_size = voxel_volume.count_all() as usize;
        tracing::info!("Voxels Count: {}", buffer_size);

        let mut storage_buffers = world.resource_mut::<Assets<ShaderStorageBuffer>>();
        let mut vertex_buffer = ShaderStorageBuffer::with_size(
            12 * size_of::<Vertex>() * buffer_size,
            RenderAssetUsages::RENDER_WORLD,
        );
        vertex_buffer.buffer_description.usage |= BufferUsages::VERTEX;
        let vertices = storage_buffers.add(vertex_buffer);

        Self { vertices }
    }
}
