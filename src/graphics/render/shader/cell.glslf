#version 150 core

uniform usampler2D t_SpriteTexture;
uniform sampler2D t_Palette;

uniform CellGlobals {
  uvec2 u_ScreenSizeInSprites;
  uvec2 u_SpriteMapDimensions;
};

in vec2 v_Uv;
flat in uint v_Index;

out vec4 IntermediateTarget;

void main() {
  vec4 t = texture(t_SpriteTexture, v_Uv);
  vec4 p = texelFetch(t_Palette, ivec2(clamp(t.x, 0, 15), v_Index), 0);
  IntermediateTarget = p;
}
