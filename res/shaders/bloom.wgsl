struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var original_texture: texture_2d<f32>;
@group(0) @binding(1) var bloom_texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let original = textureSample(original_texture, texture_sampler, in.uv);
    let bloom = textureSample(bloom_texture, texture_sampler, in.uv);

    // Adjust bloom intensity
    let bloom_intensity = 1.7;
    let result = original + bloom * bloom_intensity;

    return vec4<f32>(result.rgb, 1.0);
}
