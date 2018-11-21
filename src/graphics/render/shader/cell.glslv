#version 150 core

in vec2 a_Pos;
in vec2 a_Uv;

in vec2 a_Translate;
in uint a_Sprite;
in uint a_Index;

uniform CellGlobals {
  uvec2 u_ScreenSizeInSprites;
  uvec2 u_SpriteMapDimensions;
};

out vec2 v_Uv;
flat out uint v_Index;

void main() {
  vec2 sprite_offset = vec2(mod(a_Sprite, u_SpriteMapDimensions.x), a_Sprite / u_SpriteMapDimensions.x);
  v_Uv = a_Uv / u_SpriteMapDimensions + sprite_offset / u_SpriteMapDimensions;

  v_Index = a_Index;

  gl_Position = vec4(a_Pos * 2.0 / u_ScreenSizeInSprites + a_Translate, 0.0, 1.0);
}
