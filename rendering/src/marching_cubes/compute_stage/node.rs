use bevy_ecs::world::World;
use bevy_math::UVec3;
use bevy_render::render_graph;
use bevy_render::render_resource::{CachedPipelineState, ComputePassDescriptor, PipelineCache};
use bevy_render::renderer::RenderContext;

use super::VoxelVolumeUniform;
use super::pipeline::{MarchingCubesBindGroup, MarchingCubesPipeline};

const WORKGROUP_SIZE: u32 = 8;

#[derive(Default)]
pub struct MarchingCubesNode {
    pipeline_is_ready: bool,
}

impl render_graph::Node for MarchingCubesNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<MarchingCubesPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        if !self.pipeline_is_ready {
            match pipeline_cache.get_compute_pipeline_state(pipeline.pipeline_id) {
                CachedPipelineState::Ok(_) => {
                    self.pipeline_is_ready = true;
                }
                CachedPipelineState::Err(err) => {
                    panic!("Initializing compute_vertices.wgsl:\n{err}")
                }
                _ => {}
            }
        }
    }

    fn run<'w>(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        if !self.pipeline_is_ready {
            return Ok(());
        }

        let marching_cubes_pipeline = world.resource::<MarchingCubesPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let bind_group = world.resource::<MarchingCubesBindGroup>();
        let voxel_volume = world.resource::<VoxelVolumeUniform>();

        // TODO: Maybe add `count_dims` as for VoxelVolume
        let voxel_count = ((voxel_volume.max_bound - voxel_volume.min_bound)
            / voxel_volume.voxel_size)
            .as_uvec3();
        let workgroup_size = (voxel_count + UVec3::splat(WORKGROUP_SIZE)) / WORKGROUP_SIZE;

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        let pipeline = pipeline_cache
            .get_compute_pipeline(marching_cubes_pipeline.pipeline_id)
            .unwrap();
        pass.set_bind_group(0, &bind_group.0, &[]);
        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(workgroup_size.x, workgroup_size.y, workgroup_size.z);

        Ok(())
    }
}
