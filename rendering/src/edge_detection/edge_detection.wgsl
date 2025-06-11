#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct EdgeDetection {
    depth_threshold: vec2<f32>,
    normal_threshold: vec2<f32>,
    edge_color: vec4<f32>,
    width: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    _webgl_padding: f32,
#endif
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var depth: texture_depth_2d;
@group(0) @binding(2) var normal: texture_2d<f32>;
@group(0) @binding(3) var texture_sampler: sampler;
@group(0) @binding(4) var<uniform> settings: EdgeDetection;

@fragment
fn fragment(
    in: FullscreenVertexOutput
) -> @location(0) vec4<f32> { 
    let texture_size = vec2<f32>(textureDimensions(screen_texture));
    let texel_size = settings.width / texture_size;

    let screen_texture = textureSample(screen_texture, texture_sampler, in.uv);

#ifdef NO_DEPTH_TEXTURE_SUPPORT
    let depth_edge = 0.0;
#else
    let depth_edge = sobel_depth(in.uv, texel_size);
#endif
    let normal_edge = sobel_normal(in.uv, texel_size);

    let depth_low = settings.depth_threshold.x;
    let depth_high = settings.depth_threshold.y;
    let normal_low = settings.normal_threshold.x;
    let normal_high = settings.normal_threshold.y;
     
    // Combine the edge detection results
    let edge = smoothstep(depth_low, depth_high, depth_edge) + smoothstep(normal_low, normal_high, normal_edge);
    let final_edge = saturate(edge);

    return mix(screen_texture, settings.edge_color, final_edge);    
}

// Helper function to sample normals 
fn sample_normal(coords: vec2<f32>) -> vec3<f32> {
    let normal = textureSample(normal, texture_sampler, coords).xyz;
    return normalize(normal * 2.0 - vec3f(1.0));
}

// Helper function to sample depth 
fn sample_depth(coords: vec2<f32>) -> f32 {
    let depth = textureSample(depth, texture_sampler, coords);
    return depth;
}

// Function to apply the Sobel operator for depth
fn sobel_depth(uv: vec2<f32>, texel_size: vec2<f32>) -> f32 {
    let d00 = sample_depth(uv + texel_size * vec2<f32>(-1.0, -1.0));
    let d10 = sample_depth(uv + texel_size * vec2<f32>( 0.0, -1.0));
    let d20 = sample_depth(uv + texel_size * vec2<f32>( 1.0, -1.0));
    let d01 = sample_depth(uv + texel_size * vec2<f32>(-1.0,  0.0));
    let d21 = sample_depth(uv + texel_size * vec2<f32>( 1.0,  0.0));
    let d02 = sample_depth(uv + texel_size * vec2<f32>(-1.0,  1.0));
    let d12 = sample_depth(uv + texel_size * vec2<f32>( 0.0,  1.0));
    let d22 = sample_depth(uv + texel_size * vec2<f32>( 1.0,  1.0));

    let gx = (d20 + 2.0 * d21 + d22) - (d00 + 2.0 * d01 + d02);
    let gy = (d02 + 2.0 * d12 + d22) - (d00 + 2.0 * d10 + d20);

    return length(vec2<f32>(gx, gy)) * 8.0;
}

// Function to apply the Sobel operator for normals
fn sobel_normal(uv: vec2<f32>, texel_size: vec2<f32>) -> f32 {
    let n00 = sample_normal(uv + texel_size * vec2<f32>(-1.0, -1.0));
    let n10 = sample_normal(uv + texel_size * vec2<f32>( 0.0, -1.0));
    let n20 = sample_normal(uv + texel_size * vec2<f32>( 1.0, -1.0));
    let n01 = sample_normal(uv + texel_size * vec2<f32>(-1.0,  0.0));
    let n21 = sample_normal(uv + texel_size * vec2<f32>( 1.0,  0.0));
    let n02 = sample_normal(uv + texel_size * vec2<f32>(-1.0,  1.0));
    let n12 = sample_normal(uv + texel_size * vec2<f32>( 0.0,  1.0));
    let n22 = sample_normal(uv + texel_size * vec2<f32>( 1.0,  1.0));

    let gx = (n20 + 2.0 * n21 + n22) - (n00 + 2.0 * n01 + n02);
    let gy = (n02 + 2.0 * n12 + n22) - (n00 + 2.0 * n10 + n20);

    return length(gx) + length(gy);
}
