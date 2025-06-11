use bevy_core_pipeline::prepass::ViewPrepassTextures;
use bevy_ecs::query::QueryItem;
use bevy_ecs::world::World;
use bevy_render::extract_component::ComponentUniforms;
use bevy_render::render_graph::{NodeRunError, RenderGraphContext, ViewNode};
use bevy_render::render_resource::{
    BindGroupEntries, Operations, PipelineCache, RenderPassColorAttachment, RenderPassDescriptor,
};
use bevy_render::renderer::RenderContext;
use bevy_render::view::ViewTarget;

use super::EdgeDetectionUniform;
use super::pipeline::{EdgeDetectionPipeline, EdgeDetectionPipelineId};

#[derive(Default)]
pub struct EdgeDetectionNode;

impl ViewNode for EdgeDetectionNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static EdgeDetectionPipelineId,
        &'static ViewPrepassTextures,
        &'static EdgeDetectionUniform,
    );
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, pipeline_id, prepass_textures, _edge_detection): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let edge_detection_pipeline = world.resource::<EdgeDetectionPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let settings_uniforms = world.resource::<ComponentUniforms<EdgeDetectionUniform>>();
        let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_id.0) else {
            return Ok(());
        };

        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };
        let (Some(prepass_depth_texture), Some(prepass_normal_texture)) =
            (&prepass_textures.depth, &prepass_textures.normal)
        else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            Some("edge_detection_bind_group"),
            &edge_detection_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &prepass_depth_texture.texture.default_view,
                &prepass_normal_texture.texture.default_view,
                &edge_detection_pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("edge_detection_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
