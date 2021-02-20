[[builtin(position)]] var<out> out_pos: vec4<f32>;

[[block]]
struct ScreenGlobals {
    screen_size: vec2<u32>;
    scale_factor: vec2<f32>;
    frame_counter: u32;
    elapsed_time: f32;
};


[[location(0)]] var<in> in_pos: vec2<f32>;
[[location(1)]] var<in> in_uv: vec2<f32>;

[[location(0)]] var<out> out_uv: vec2<f32>;

[[group(1), binding(0)]] var screen_globals: ScreenGlobals;

[[stage(vertex)]]
fn vs_main() {
  out_uv = in_uv;
  out_pos = vec4<f32>(in_pos * screen_globals.scale_factor, 0.0, 1.0);
}

[[location(0)]] var<in> in_uv: vec2<f32>;

[[location(0)]] var<out> out_color: vec4<f32>;

[[group(0), binding(0)]] var screen_texture: texture_2d<f32>;
[[group(0), binding(1)]] var screen_sampler: sampler;
[[group(1), binding(0)]] var screen_globals: ScreenGlobals;

[[stage(fragment)]]
fn fs_main() {
    out_color = textureSample(screen_texture, screen_sampler, in_uv);
    //var color: vec4<f32> = textureSample(screen_texture, screen_sampler, in_uv);
    //out_color = textureSample(screen_texture, screen_sampler, in_uv) * (sin(in_uv.y*1440.0) * 0.5 + 0.5);
    //out_color = out_color * (0.85 + sin(60.0 * screen_globals.elapsed_time) * 0.25);
}
