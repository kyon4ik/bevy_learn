use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Commands, Res};
use bevy_ecs::world::{FromWorld, World};
use bevy_render::render_asset::RenderAssets;
use bevy_render::render_resource::binding_types::{storage_buffer_sized, uniform_buffer_sized};
use bevy_render::render_resource::{
    BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
    ComputePipelineDescriptor, PipelineCache, ShaderStages, ShaderType,
};
use bevy_render::renderer::RenderDevice;
use bevy_render::storage::GpuShaderStorageBuffer;

use crate::marching_cubes::MarchingCubesBuffers;

use super::{COMPUTE_STAGE_SHADER_HANDLE, VoxelVolumeBuffer, VoxelVolumeUniform};

#[derive(Resource)]
pub struct MarchingCubesBindGroup(pub(crate) BindGroup);

pub fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<MarchingCubesPipeline>,
    marching_cubes_buffers: Res<MarchingCubesBuffers>,
    gpu_buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
    settings_buffer: Res<VoxelVolumeBuffer>,
    render_device: Res<RenderDevice>,
) {
    let vertices = gpu_buffers
        .get(marching_cubes_buffers.vertices.id())
        .unwrap();

    let bind_group = render_device.create_bind_group(
        Some("marching_cubes_bind_group"),
        &pipeline.bind_group_layout,
        &BindGroupEntries::sequential((
            vertices.buffer.as_entire_buffer_binding(),
            &settings_buffer.buffer,
        )),
    );
    commands.insert_resource(MarchingCubesBindGroup(bind_group));
}

#[derive(Resource)]
pub struct MarchingCubesPipeline {
    pub(crate) bind_group_layout: BindGroupLayout,
    pub(crate) pipeline_id: CachedComputePipelineId,
}

impl FromWorld for MarchingCubesPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = render_device.create_bind_group_layout(
            "marching_cubes_bind_croup",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer_sized(false, None),
                    uniform_buffer_sized(false, Some(VoxelVolumeUniform::min_size())),
                ),
            ),
        );
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: COMPUTE_STAGE_SHADER_HANDLE,
            shader_defs: vec![],
            entry_point: "compute_vertices".into(),
            zero_initialize_workgroup_memory: false,
        });

        Self {
            bind_group_layout,
            pipeline_id,
        }
    }
}
