use bevy_app::{App, Plugin};
use bevy_asset::{AssetId, Handle, load_internal_asset, weak_handle};
use bevy_core_pipeline::core_3d::{
    CORE_3D_DEPTH_FORMAT, Opaque3d, Opaque3dBatchSetKey, Opaque3dBinKey,
};
use bevy_ecs::component::{Component, Tick};
use bevy_ecs::query::ROQueryItem;
use bevy_ecs::resource::Resource;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::system::lifetimeless::SRes;
use bevy_ecs::system::{Local, Query, Res, ResMut, SystemParamItem};
use bevy_ecs::world::FromWorld;
use bevy_image::BevyDefault;
use bevy_pbr::{MeshPipeline, MeshPipelineKey, MeshPipelineViewLayoutKey, SetMeshViewBindGroup};
use bevy_render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy_render::mesh::{Mesh, PrimitiveTopology, VertexBufferLayout, VertexFormat};
use bevy_render::render_asset::RenderAssets;
use bevy_render::render_phase::{
    AddRenderCommand, BinnedRenderPhaseType, DrawFunctions, InputUniformIndex, PhaseItem,
    RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewBinnedRenderPhases,
};
use bevy_render::render_resource::{
    ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, Face, FragmentState,
    MultisampleState, PipelineCache, PrimitiveState, RenderPipelineDescriptor, Shader,
    SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat, VertexAttribute,
    VertexState, VertexStepMode,
};
use bevy_render::storage::GpuShaderStorageBuffer;
use bevy_render::view::{ExtractedView, Msaa, RenderVisibleEntities, ViewTarget, VisibilityClass};
use bevy_render::{Render, RenderApp, RenderSet, view};

use super::{MarchingCubesBuffers, Vertex};

pub struct VoxelRenderedPlugin;

// structs
#[derive(Clone, Component, ExtractComponent)]
#[require(VisibilityClass)]
#[component(on_add = view::add_visibility_class::<VoxeledRendered>)]
pub struct VoxeledRendered;

#[derive(Resource, FromWorld)]
pub struct VoxelRenderedPipeline {
    mesh_pipeline: MeshPipeline,
}

struct DrawVoxeled;

type DrawVoxeledCommands = (SetItemPipeline, SetMeshViewBindGroup<0>, DrawVoxeled);

pub const DISPLAY_STAGE_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("65b1d237-3e83-4d22-8097-2bb33a3462ae");

impl Plugin for VoxelRenderedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<VoxeledRendered>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_render_command::<Opaque3d, DrawVoxeledCommands>();
        render_app.add_systems(Render, queue_voxel_rendered_phase.in_set(RenderSet::Queue));
    }

    fn finish(&self, app: &mut App) {
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
            .init_resource::<VoxelRenderedPipeline>()
            .init_resource::<SpecializedRenderPipelines<VoxelRenderedPipeline>>();
    }
}

impl<P: PhaseItem> RenderCommand<P> for DrawVoxeled {
    type Param = (
        SRes<MarchingCubesBuffers>,
        SRes<RenderAssets<GpuShaderStorageBuffer>>,
    );

    type ViewQuery = ();

    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let marching_cubes_buffers = param.0.into_inner();
        let gpu_storage_buffers = param.1.into_inner();

        let vertices = gpu_storage_buffers
            .get(marching_cubes_buffers.vertices.id())
            .unwrap();

        pass.set_vertex_buffer(0, vertices.buffer.slice(..));

        let num = vertices.buffer.size() / size_of::<Vertex>() as u64;
        pass.draw(0..num as u32, 0..1);

        RenderCommandResult::Success
    }
}

impl SpecializedRenderPipeline for VoxelRenderedPipeline {
    type Key = MeshPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("voxel_rendered_pipeline".into()),
            layout: vec![
                self.mesh_pipeline
                    .get_view_layout(MeshPipelineViewLayoutKey::from(key))
                    .clone(),
            ],
            push_constant_ranges: vec![],
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
                    format: if key.contains(MeshPipelineKey::HDR) {
                        ViewTarget::TEXTURE_FORMAT_HDR
                    } else {
                        TextureFormat::bevy_default()
                    },
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: key.primitive_topology(),
                cull_mode: Some(Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            zero_initialize_workgroup_memory: false,
        }
    }
}

fn queue_voxel_rendered_phase(
    pipeline_cache: Res<PipelineCache>,
    voxel_rendered_pipeline: Res<VoxelRenderedPipeline>,
    mut opaque_render_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    opaque_draw_functions: Res<DrawFunctions<Opaque3d>>,
    mut specialized_render_pipelines: ResMut<SpecializedRenderPipelines<VoxelRenderedPipeline>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities, &Msaa)>,
    mut next_tick: Local<Tick>,
) {
    let draw_voxel_rendered = opaque_draw_functions.read().id::<DrawVoxeledCommands>();

    for (view, view_visible_entities, msaa) in views.iter() {
        let Some(opaque_phase) = opaque_render_phases.get_mut(&view.retained_view_entity) else {
            continue;
        };

        for &entity in view_visible_entities.get::<VoxeledRendered>().iter() {
            let pipeline_id = specialized_render_pipelines.specialize(
                &pipeline_cache,
                &voxel_rendered_pipeline,
                MeshPipelineKey::from_msaa_samples(msaa.samples())
                    | MeshPipelineKey::from_hdr(view.hdr)
                    | MeshPipelineKey::from_primitive_topology(PrimitiveTopology::TriangleList),
            );

            let this_tick = next_tick.get() + 1;
            next_tick.set(this_tick);

            opaque_phase.add(
                Opaque3dBatchSetKey {
                    draw_function: draw_voxel_rendered,
                    pipeline: pipeline_id,
                    material_bind_group_index: None,
                    vertex_slab: Default::default(),
                    index_slab: None,
                    lightmap_slab: None,
                },
                Opaque3dBinKey {
                    asset_id: AssetId::<Mesh>::invalid().untyped(),
                },
                entity,
                InputUniformIndex::default(),
                BinnedRenderPhaseType::NonMesh,
                *next_tick,
            );
        }
    }
}
