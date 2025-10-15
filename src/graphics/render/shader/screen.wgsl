struct ScreenGlobals {
    screen_size: vec2<f32>,
    screen_texture_dimensions: vec2<f32>,
    scale_factor: vec2<f32>,
    frame_counter: u32,
    elapsed_time: f32,
}

struct ScreenVertexOutput {
  @builtin(position) pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) texel: vec2<f32>,
}

struct ScreenVertexInput {
  @location(0) pos: vec2<f32>,
  @location(1) uv: vec2<f32>,
}

@group(1) @binding(0) var<uniform> screen_globals: ScreenGlobals;

@vertex
fn vs_main(in: ScreenVertexInput) -> ScreenVertexOutput {
  var out: ScreenVertexOutput;
  out.uv = in.uv;
  out.pos = vec4<f32>(in.pos * screen_globals.scale_factor, 0.0, 1.0);
  out.texel = in.uv * screen_globals.screen_texture_dimensions;
  return out;
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;

@fragment
fn fs_main(in: ScreenVertexOutput) -> @location(0) vec4<f32> {
  let slope = 1.0 / fwidth(in.texel);
  let subtexel = fract(in.texel);
  let scaled = clamp(slope * subtexel, vec2(0.0, 0.0), vec2(0.5, 0.5)) +
               clamp(slope * (subtexel - 1.0) + 0.5, vec2(0.0, 0.0), vec2(0.5, 0.5));
  let clamped_uv = (floor(in.texel) + scaled) / screen_globals.screen_texture_dimensions;
  return textureSample(screen_texture, screen_sampler, clamped_uv);
}
