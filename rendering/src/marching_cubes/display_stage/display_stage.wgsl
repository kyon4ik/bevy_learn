#import bevy_pbr::view_transformations::position_world_to_clip;

struct Vertex {
    @builtin(vertex_index) index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // @location(0) color: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = position_world_to_clip(vertex.position);
    // out.color = vec3<f32>(f32(vertex.index & 1), f32((vertex.index & 2) >> 1),
    // f32((vertex.index & 4) >> 2));
    return out;
}

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0);
}
