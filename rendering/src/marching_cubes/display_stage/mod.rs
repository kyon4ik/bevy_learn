use bevy_app::{App, Plugin};
use bevy_asset::{Handle, load_internal_asset, weak_handle};
use bevy_core_pipeline::core_3d::CORE_3D_DEPTH_FORMAT;
use bevy_core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy_ecs::query::QueryItem;
use bevy_ecs::resource::Resource;
use bevy_ecs::world::{FromWorld, World};
use bevy_image::BevyDefault;
use bevy_render::RenderApp;
use bevy_render::mesh::{VertexBufferLayout, VertexFormat};
use bevy_render::render_asset::RenderAssets;
use bevy_render::render_graph::{
    NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
};
use bevy_render::render_resource::binding_types::uniform_buffer;
use bevy_render::render_resource::{
    BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
    ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, FragmentState,
    MultisampleState, Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipelineDescriptor, Shader, ShaderStages, StoreOp, TextureFormat,
    VertexAttribute, VertexState, VertexStepMode,
};
use bevy_render::renderer::{RenderContext, RenderDevice};
use bevy_render::storage::GpuShaderStorageBuffer;
use bevy_render::view::{
    ViewDepthTexture, ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms,
};

use super::{MarchingCubesBuffers, Vertex};

pub struct MarchingCubesDisplayPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct MarchingCubesDisplayLabel;

pub const DISPLAY_STAGE_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("65b1d237-3e83-4d22-8097-2bb33a3462ae");

impl Plugin for MarchingCubesDisplayPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            DISPLAY_STAGE_SHADER_HANDLE,
            "display_stage.wgsl",
            Shader::from_wgsl
        );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_render_graph_node::<ViewNodeRunner<MarchingCubesDisplayNode>>(
                Core3d,
                MarchingCubesDisplayLabel,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::Tonemapping,
                    MarchingCubesDisplayLabel,
                    Node3d::EndMainPassPostProcessing,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<MarchingCubesDisplayPipeline>();
    }
}

#[derive(Default)]
pub struct MarchingCubesDisplayNode;

#[derive(Resource)]
pub struct MarchingCubesDisplayPipeline {
    layout: BindGroupLayout,
    pipeline_id: CachedRenderPipelineId,
}

impl ViewNode for MarchingCubesDisplayNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewDepthTexture,
        &'static ViewUniformOffset,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, view_depth, view_uniform_offset): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let display_pipeline = world.resource::<MarchingCubesDisplayPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let marching_cubes_buffers = world.resource::<MarchingCubesBuffers>();
        let gpu_buffers = world.resource::<RenderAssets<GpuShaderStorageBuffer>>();
        let view_uniforms = world.resource::<ViewUniforms>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(display_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let Some(vertex_buffer) = gpu_buffers.get(marching_cubes_buffers.vertices.id()) else {
            return Ok(());
        };

        let Some(view_binding) = view_uniforms.uniforms.binding() else {
            return Ok(());
        };

        let bind_group = render_context.render_device().create_bind_group(
            Some("marching_cubes_display_bind_group"),
            &display_pipeline.layout,
            &BindGroupEntries::single(view_binding),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("marching_cubes_display_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: view_target.post_process_write().destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: Some(view_depth.get_attachment(StoreOp::Store)),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.buffer.slice(..));
        render_pass.set_bind_group(0, &bind_group, &[view_uniform_offset.offset]);

        let num_vertices = vertex_buffer.buffer.size() / size_of::<Vertex>() as u64;
        render_pass.draw(0..num_vertices as u32, 0..1);

        Ok(())
    }
}

impl FromWorld for MarchingCubesDisplayPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            "marching_cubes_display_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX,
                uniform_buffer::<ViewUniform>(true),
            ),
        );

        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("marching_cubes_display_pipeline".into()),
                    layout: vec![layout.clone()],
                    vertex: VertexState {
                        shader: DISPLAY_STAGE_SHADER_HANDLE,
                        shader_defs: vec![],
                        entry_point: "vertex".into(),
                        buffers: vec![VertexBufferLayout {
                            array_stride: size_of::<Vertex>() as u64,
                            step_mode: VertexStepMode::Vertex,
                            attributes: vec![VertexAttribute {
                                format: VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 0,
                            }],
                        }],
                    },
                    fragment: Some(FragmentState {
                        shader: DISPLAY_STAGE_SHADER_HANDLE,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::bevy_default(),
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState {
                        // cull_mode: None,
                        ..Default::default()
                    },
                    depth_stencil: Some(DepthStencilState {
                        format: CORE_3D_DEPTH_FORMAT,
                        depth_write_enabled: false,
                        depth_compare: CompareFunction::Greater,
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                    zero_initialize_workgroup_memory: false,
                });

        Self {
            layout,
            pipeline_id,
        }
    }
}
