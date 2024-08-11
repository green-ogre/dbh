struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(input_texture));
    let pixel = 1.0 / tex_size;

    var result = vec3<f32>(0.0);
    let weight_0 = 0.227027;
    let weight_1 = 0.1945946;
    let weight_2 = 0.1216216;
    let weight_3 = 0.054054;
    let weight_4 = 0.016216;

    result += textureSample(input_texture, input_sampler, in.uv).rgb * weight_0;

    let blur_direction = vec2<f32>(1.0, 0.0);

    let offset_1 = blur_direction * pixel;
    let offset_2 = blur_direction * 2.0 * pixel;
    let offset_3 = blur_direction * 3.0 * pixel;
    let offset_4 = blur_direction * 4.0 * pixel;

    result += textureSample(input_texture, input_sampler, in.uv + offset_1).rgb * weight_1;
    result += textureSample(input_texture, input_sampler, in.uv - offset_1).rgb * weight_1;

    result += textureSample(input_texture, input_sampler, in.uv + offset_2).rgb * weight_2;
    result += textureSample(input_texture, input_sampler, in.uv - offset_2).rgb * weight_2;

    result += textureSample(input_texture, input_sampler, in.uv + offset_3).rgb * weight_3;
    result += textureSample(input_texture, input_sampler, in.uv - offset_3).rgb * weight_3;

    result += textureSample(input_texture, input_sampler, in.uv + offset_4).rgb * weight_4;
    result += textureSample(input_texture, input_sampler, in.uv - offset_4).rgb * weight_4;

    return vec4<f32>(result, 1.0);
}
