[[builtin(position)]] var<out> out_pos: vec4<f32>;

[[location(0)]] var<in> in_pos: vec2<f32>;
[[location(1)]] var<in> in_uv: vec2<f32>;
[[location(2)]] var<in> in_translate: vec2<f32>;
[[location(3)]] var<in> in_cell_coords: vec2<u32>;
[[location(4)]] var<in> in_sprite: u32;
[[location(5)]] var<in> in_index: u32;

[[location(0), interpolate(flat)]] var<out> out_index: u32;
[[location(1)]] var<out> out_uv: vec2<f32>;
[[location(2), interpolate(flat)]] var<out> out_cell_coords: vec2<u32>;

[[block]]
struct CellGlobals {
  screen_size_in_sprites: vec2<u32>;
  sprite_map_dimensions: vec2<u32>;
  palette_texture_dimensions: vec2<u32>;
};

[[group(0), binding(0)]] var<uniform> cell_globals: CellGlobals;

[[stage(vertex)]]
fn vs_main() {
    var sprite_offset: vec2<f32> = vec2<f32>(
        f32(in_sprite % cell_globals.sprite_map_dimensions.x),
        f32(in_sprite / cell_globals.sprite_map_dimensions.x));

    out_uv =
        in_uv / vec2<f32>(cell_globals.sprite_map_dimensions)
        + sprite_offset / vec2<f32>(cell_globals.sprite_map_dimensions);

    out_index = in_index;
    out_cell_coords = in_cell_coords;

    out_pos = vec4<f32>(
        in_pos * 2.0 / vec2<f32>(cell_globals.screen_size_in_sprites) + in_translate,
        0.0,
        1.0);
}

[[location(0), interpolate(flat)]] var<in> in_index: u32;
[[location(1)]] var<in> in_uv: vec2<f32>;
[[location(2), interpolate(flat)]] var<in> in_cell_coords: vec2<u32>;

[[location(0)]] var<out> out_color: vec4<f32>;

[[group(0), binding(0)]] var<uniform> cell_globals: CellGlobals;

[[group(1), binding(0)]] var sprite_texture: texture_2d<u32>;
[[group(1), binding(1)]] var sprite_texture_sampler: sampler;
[[group(1), binding(2)]] var palette_texture: texture_2d<f32>;
[[group(1), binding(3)]] var palette_texture_sampler: sampler;

[[stage(fragment)]]
fn fs_main() {
    var t: vec4<u32> = textureSample(sprite_texture, sprite_texture_sampler, in_uv);
    var pc: vec2<u32> = vec2<u32>(in_cell_coords.x * 16 + clamp(t.x, 0, 15), in_cell_coords.y);
    // TODO: Replace textureSample() with textureLoad() after the latter is supported by Naga.
    var pcf: vec2<f32> = vec2<f32>(pc) + vec2<f32>(0.5);
    var pcuv: vec2<f32> = pcf / vec2<f32>(cell_globals.palette_texture_dimensions);
    out_color = textureSample(palette_texture, palette_texture_sampler, pcuv);
}
