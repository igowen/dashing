[[builtin(position)]] var<out> out_pos: vec4<f32>;

[[location(0)]] var<in> in_pos: vec2<f32>;
[[location(1)]] var<in> in_uv: vec2<f32>;
[[location(2)]] var<in> in_translate: vec2<f32>;
[[location(3)]] var<in> in_sprite_pos: vec2<f32>;
[[location(4)]] var<in> in_sprite: u32;
[[location(5)]] var<in> in_index: u32;

[[location(0), interpolate(flat)]] var<out> out_index: u32;
[[location(1)]] var<out> out_uv: vec2<f32>;
[[location(2), interpolate(flat)]] var<out> out_sprite_pos: vec2<f32>;

[[block]]
struct CellGlobals {
  screen_size_in_sprites: vec2<u32>;
  sprite_map_dimensions: vec2<u32>;
};

[[group(0), binding(0)]] var<uniform> cell_globals: CellGlobals;


fn hsv2rgb(h: f32, s: f32, v: f32) -> vec4<f32> {
    var hh: f32 = h % 360.0;
    if (h < 0.0) {
      hh = hh + 360.0;
    }
    hh = hh / 60.0;

    var ss: f32 = clamp(s, 0.0, 1.0);
    var vv: f32 = clamp(v, 0.0, 1.0);

    var chroma: f32 = vv * ss;
    var x: f32 = chroma * (1.0 - abs(hh % 2.0 - 1.0));

    var m: f32 = vv - chroma;

    var i: f32 = chroma + m;
    var j: f32 = x + m;
    var k: f32 = m;

    var hhi: u32 = u32(hh);
    if (hhi == 0) {
      return vec4<f32>(i, j, k, 1.0);
    }
    if (hhi == 1) {
      return vec4<f32>(j, i, k, 1.0);
    }
    if (hhi == 2) {
      return vec4<f32>(k, i, j, 1.0);
    }
    if (hhi == 3) {
      return vec4<f32>(k, j, i, 1.0);
    }
    if (hhi == 4) {
      return vec4<f32>(j, k, i, 1.0);
    }
    return vec4<f32>(i, k, j, 1.0);
}

[[stage(vertex)]]
fn vs_main() {
    var sprite_offset: vec2<f32> = vec2<f32>(
        f32(in_sprite % cell_globals.sprite_map_dimensions.x),
        f32(in_sprite / cell_globals.sprite_map_dimensions.x));

    out_uv =
        in_uv / vec2<f32>(cell_globals.sprite_map_dimensions)
        + sprite_offset / vec2<f32>(cell_globals.sprite_map_dimensions);

    out_index = in_index;
    out_sprite_pos = in_sprite_pos;

    out_pos = vec4<f32>(
        in_pos * 2.0 / vec2<f32>(cell_globals.screen_size_in_sprites) + in_translate,
        0.0,
        1.0);
}

[[location(0), interpolate(flat)]] var<in> in_index: u32;
[[location(1)]] var<in> in_uv: vec2<f32>;
[[location(2), interpolate(flat)]] var<in> in_sprite_pos: vec2<f32>;

[[location(0)]] var<out> out_color: vec4<f32>;

[[group(1), binding(0)]] var sprite_texture: texture_2d<u32>;
[[group(1), binding(1)]] var sprite_texture_sampler: sampler;

[[stage(fragment)]]
fn fs_main() {
    var tex: vec4<u32> = textureSample(sprite_texture, sprite_texture_sampler, in_uv);
    var idx: u32 = tex.x;
    var v: f32;
    if (idx > 0) {
      v = 1.0;
    } else {
      v = 0.0;
    }
    //var tex: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    out_color = hsv2rgb(f32(in_index)*5.0, 1.0, v);
}
