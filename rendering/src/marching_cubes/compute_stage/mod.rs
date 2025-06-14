use bevy_app::{App, Plugin};
use bevy_asset::{Handle, load_internal_asset, weak_handle};
use bevy_ecs::resource::Resource;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::system::{Res, ResMut};
use bevy_math::bounding::Aabb3d;
use bevy_math::{UVec3, Vec3};
use bevy_render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy_render::render_graph::{RenderGraph, RenderLabel};
use bevy_render::render_resource::{Shader, ShaderType, UniformBuffer};
use bevy_render::renderer::{RenderDevice, RenderQueue};
use bevy_render::{Render, RenderApp, RenderSet};

pub mod node;
pub mod pipeline;

pub struct MarchingCubesComputePlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct MarchingCubesComputeLabel;

pub const COMPUTE_STAGE_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("f2b936a3-3f56-4386-b58d-76eb65df3058");

impl Plugin for MarchingCubesComputePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            COMPUTE_STAGE_SHADER_HANDLE,
            "compute_stage.wgsl",
            Shader::from_wgsl
        );

        app.init_resource::<VoxelVolume>();
        app.add_plugins(ExtractResourcePlugin::<VoxelVolumeUniform>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<VoxelVolumeBuffer>();
        render_app.add_systems(
            Render,
            (
                prepare_voxel_volume_buffer.in_set(RenderSet::PrepareResources),
                pipeline::prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
            ),
        );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(
            MarchingCubesComputeLabel,
            node::MarchingCubesNode::default(),
        );
        render_graph.add_node_edge(
            MarchingCubesComputeLabel,
            bevy_render::graph::CameraDriverLabel,
        );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<pipeline::MarchingCubesPipeline>();
    }
}

impl VoxelVolume {
    #[inline]
    pub fn count_dims(&self) -> UVec3 {
        ((self.aabb.max - self.aabb.min) / self.voxel_size).as_uvec3()
    }

    #[inline]
    pub fn count_all(&self) -> u32 {
        self.count_dims().element_product()
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct VoxelVolume {
    pub aabb: Aabb3d,
    pub voxel_size: f32,
}

impl Default for VoxelVolume {
    fn default() -> Self {
        Self {
            aabb: Aabb3d::new(Vec3::ZERO, Vec3::ONE),
            voxel_size: 0.25,
        }
    }
}

impl ExtractResource for VoxelVolumeUniform {
    type Source = VoxelVolume;

    fn extract_resource(source: &Self::Source) -> Self {
        Self {
            min_bound: source.aabb.min.into(),
            max_bound: source.aabb.max.into(),
            voxel_size: source.voxel_size,
        }
    }
}

#[derive(Resource, ShaderType, Clone, Default)]
pub struct VoxelVolumeUniform {
    min_bound: Vec3,
    max_bound: Vec3,
    voxel_size: f32,
}

#[derive(Resource, Default)]
pub struct VoxelVolumeBuffer {
    pub buffer: UniformBuffer<VoxelVolumeUniform>,
}

fn prepare_voxel_volume_buffer(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut voxel_volume_buffer: ResMut<VoxelVolumeBuffer>,
    voxel_volume: Res<VoxelVolumeUniform>,
) {
    let buffer = voxel_volume_buffer.buffer.get_mut();
    buffer.clone_from(&voxel_volume);

    voxel_volume_buffer
        .buffer
        .write_buffer(&render_device, &render_queue);
}
