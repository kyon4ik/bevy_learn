use bevy_app::{App, Plugin};
use bevy_asset::{Handle, load_internal_asset, weak_handle};
use bevy_color::{Color, LinearRgba};
use bevy_core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy_core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use bevy_ecs::component::Component;
use bevy_ecs::query::{QueryItem, With};
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_math::Vec2;
use bevy_render::camera::Camera;
use bevy_render::extract_component::{
    ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin,
};
use bevy_render::render_graph::{RenderGraphApp, RenderLabel, ViewNodeRunner};
use bevy_render::render_resource::{Shader, ShaderType, SpecializedRenderPipelines};
use bevy_render::{Render, RenderApp, RenderSet};

pub mod node;
pub mod pipeline;

#[derive(Component, Clone)]
#[require(DepthPrepass, NormalPrepass)]
pub struct EdgeDetection {
    pub edge_color: Color,
    pub width: f32,
    pub depth_threshold: Vec2,
    pub normal_threshold: Vec2,
    pub final_threshold: f32,
}

impl Default for EdgeDetection {
    fn default() -> Self {
        Self {
            edge_color: Color::BLACK,
            width: 1.5,
            depth_threshold: Vec2::new(0.0, 0.02),
            normal_threshold: Vec2::new(0.5, 5.0),
            final_threshold: 0.5,
        }
    }
}

impl ExtractComponent for EdgeDetection {
    type QueryData = &'static Self;
    type QueryFilter = With<Camera>;
    type Out = EdgeDetectionUniform;

    fn extract_component(item: QueryItem<Self::QueryData>) -> Option<Self::Out> {
        Some(EdgeDetectionUniform {
            depth_threshold: item.depth_threshold,
            normal_threshold: item.normal_threshold,
            edge_color: item.edge_color.to_linear(),
            width: item.width,
            final_threshold: item.final_threshold,
        })
    }
}

#[derive(Component, ShaderType, Clone)]
pub struct EdgeDetectionUniform {
    depth_threshold: Vec2,
    normal_threshold: Vec2,
    edge_color: LinearRgba,
    width: f32,
    final_threshold: f32,
}

pub const EDGE_DETECTION_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("f6bf5831-e8f1-4fb4-a616-013c88b72d3c");

pub struct EdgeDetectionPlugin;
impl Plugin for EdgeDetectionPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            EDGE_DETECTION_SHADER_HANDLE,
            "edge_detection.wgsl",
            Shader::from_wgsl
        );
        app.add_plugins((
            ExtractComponentPlugin::<EdgeDetection>::default(),
            UniformComponentPlugin::<EdgeDetectionUniform>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedRenderPipelines<pipeline::EdgeDetectionPipeline>>()
            .add_systems(
                Render,
                pipeline::prepare_edge_detection_pipelines.in_set(RenderSet::Prepare),
            );

        render_app
            .add_render_graph_node::<ViewNodeRunner<node::EdgeDetectionNode>>(
                Core3d,
                EdgeDetectionLabel,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::PostProcessing,
                    EdgeDetectionLabel,
                    Node3d::EndMainPassPostProcessing,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<pipeline::EdgeDetectionPipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct EdgeDetectionLabel;
