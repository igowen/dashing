struct CellVertexInput {
  [[location(0)]] pos: vec2<f32>;
  [[location(1)]] uv: vec2<f32>;
  [[location(2)]] translate: vec2<f32>;
  [[location(3)]] cell_coords: vec2<u32>;
  [[location(4)]] sprite: u32;
  [[location(5)]] index: u32;
};

struct CellVertexOutput {
  [[builtin(position)]] pos: vec4<f32>;
  [[location(0), interpolate(flat)]] index: u32;
  [[location(1)]] uv: vec2<f32>;
  [[location(2), interpolate(flat)]] cell_coords: vec2<u32>;
};


[[block]]
struct CellGlobals {
  screen_size_in_sprites: vec2<u32>;
  sprite_map_dimensions: vec2<u32>;
  sprite_texture_dimensions: vec2<u32>;
  sprite_dimensions: vec2<u32>;
  palette_texture_dimensions: vec2<u32>;
};

[[group(0), binding(0)]] var<uniform> cell_globals: CellGlobals;

[[stage(vertex)]]
fn vs_main(in: CellVertexInput) -> CellVertexOutput {
    var out: CellVertexOutput;
    var sprite_offset: vec2<f32> = vec2<f32>(
        f32(in.sprite % cell_globals.sprite_map_dimensions.x),
        f32(in.sprite / cell_globals.sprite_map_dimensions.x));

    out.uv =
        in.uv / vec2<f32>(cell_globals.sprite_map_dimensions)
        + sprite_offset / vec2<f32>(cell_globals.sprite_map_dimensions);

    out.index = in.index;
    out.cell_coords = in.cell_coords;

    out.pos = vec4<f32>(
        in.pos * 2.0 / vec2<f32>(cell_globals.screen_size_in_sprites) + in.translate,
        0.0,
        1.0);

    return out;
}

struct CellFragmentInput {
  [[location(0), interpolate(flat)]] index: u32;
  [[location(1)]] uv: vec2<f32>;
  [[location(2), interpolate(flat)]] cell_coords: vec2<u32>;
};

struct CellFragmentOutput {
  [[location(0)]] color: vec4<f32>;
};

[[group(0), binding(0)]] var<uniform> cell_globals: CellGlobals;

[[group(1), binding(0)]] var sprite_texture: texture_2d<u32>;
[[group(1), binding(1)]] var palette_texture: texture_3d<f32>;

[[stage(fragment)]]
fn fs_main(in: CellFragmentInput) -> CellFragmentOutput {
    var out: CellFragmentOutput;
    // The "color" here is the index into the palette for this cell (0-15).
    var t: vec4<u32> = textureLoad(
        sprite_texture,
        vec2<i32>(i32(floor(in.uv.x * f32(cell_globals.sprite_texture_dimensions.x))),
                  i32(floor(in.uv.y * f32(cell_globals.sprite_texture_dimensions.y)))),
        0);
    out.color = textureLoad(
        palette_texture,
        vec3<i32>(i32(clamp(t.x, 0u32, 15u32)),
                  i32(in.cell_coords.x),
                  i32(in.cell_coords.y)),
        0);

    return out;
}
