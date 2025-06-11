use bevy_core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state;
use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::With,
    resource::Resource,
    system::{Commands, Query, Res, ResMut},
    world::{FromWorld, World},
};
use bevy_image::BevyDefault as _;
use bevy_render::{
    render_resource::{
        BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState,
        ColorWrites, FragmentState, MultisampleState, PipelineCache, PrimitiveState,
        RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
        ShaderType, SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat,
        TextureSampleType,
        binding_types::{sampler, texture_2d, texture_depth_2d, uniform_buffer_sized},
    },
    renderer::RenderDevice,
    view::{ExtractedView, Msaa, ViewTarget},
};

use super::{EDGE_DETECTION_SHADER_HANDLE, EdgeDetectionUniform};

#[derive(Resource)]
pub struct EdgeDetectionPipeline {
    pub(crate) sampler: Sampler,
    pub(crate) layout: BindGroupLayout,
}

impl EdgeDetectionPipeline {
    pub(crate) fn new(render_device: &RenderDevice) -> Self {
        let mb_layout = &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                // View target (read)
                texture_2d(TextureSampleType::Float { filterable: true }),
                // Depth
                texture_depth_2d(),
                // Normal Vectors
                texture_2d(TextureSampleType::Float { filterable: true }),
                // Linear Sampler
                sampler(SamplerBindingType::Filtering),
                // Edge detection settings uniform input
                uniform_buffer_sized(false, Some(EdgeDetectionUniform::min_size())),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());
        let layout = render_device.create_bind_group_layout("edge_detection_layout", mb_layout);

        Self { sampler, layout }
    }
}

impl FromWorld for EdgeDetectionPipeline {
    fn from_world(render_world: &mut World) -> Self {
        let render_device = render_world.resource::<RenderDevice>().clone();
        EdgeDetectionPipeline::new(&render_device)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct EdgeDetectionPipelineKey {
    hdr: bool,
}

impl SpecializedRenderPipeline for EdgeDetectionPipeline {
    type Key = EdgeDetectionPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("edge_detection_pipeline".into()),
            layout: vec![self.layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: EDGE_DETECTION_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: if key.hdr {
                        ViewTarget::TEXTURE_FORMAT_HDR
                    } else {
                        TextureFormat::bevy_default()
                    },
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        }
    }
}

#[derive(Component)]
pub struct EdgeDetectionPipelineId(pub CachedRenderPipelineId);

pub(crate) fn prepare_edge_detection_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<EdgeDetectionPipeline>>,
    pipeline: Res<EdgeDetectionPipeline>,
    views: Query<(Entity, &ExtractedView, &Msaa), With<EdgeDetectionUniform>>,
) {
    for (entity, view, msaa) in &views {
        if *msaa != Msaa::Off {
            tracing::error!(
                "Edge detection is being used which requires Msaa::Off, but Msaa is currently set to Msaa::{:?}",
                *msaa
            );
            return;
        }

        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &pipeline,
            EdgeDetectionPipelineKey { hdr: view.hdr },
        );

        commands
            .entity(entity)
            .insert(EdgeDetectionPipelineId(pipeline_id));
    }
}
