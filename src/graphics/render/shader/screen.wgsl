[[builtin(position)]] var<out> out_pos: vec4<f32>;

[[location(0)]] var<in> in_pos: vec2<f32>;
[[location(1)]] var<in> in_uv: vec2<f32>;

[[location(0)]] var<out> out_uv: vec2<f32>;

[[stage(vertex)]]
fn vs_main() {
  out_uv = in_uv;
  out_pos = vec4<f32>(in_pos, 0.0, 1.0);
}

[[location(0)]] var<in> in_uv: vec2<f32>;

[[location(0)]] var<out> out_color: vec4<f32>;

[[group(0), binding(0)]] var screen_texture: texture_2d<f32>;
[[group(0), binding(1)]] var screen_sampler: sampler;

[[stage(fragment)]]
fn fs_main() {
    out_color = textureSample(screen_texture, screen_sampler, in_uv);
}
