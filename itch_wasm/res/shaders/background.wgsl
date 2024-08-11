@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct BackgroundUniform {
  time: f32,
  scroll_speed: f32,
}

@group(0) @binding(2)
var<uniform> background: BackgroundUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(texture, texture_sampler, in.uv * 2.0 + vec2(background.time, background.time) * background.scroll_speed);
    let gray = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    let gray_scale = vec4<f32>(gray, gray, gray, color.a);
    return gray_scale * vec4(23.0 / 255.0, 0.0 / 255.0, 29.0 / 255.0, 1.0);
}
