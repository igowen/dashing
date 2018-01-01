#version 150 core

uniform sampler2D t_SpriteTexture;
uniform CellGlobals {
  vec2 u_ScreenCharDim;
  vec2 u_FontCharDim;
};

in vec2 v_Uv;
in vec4 v_FgColor;
in vec4 v_BgColor;

out vec4 IntermediateTarget;

void main() {
  vec4 t = texture(t_SpriteTexture, v_Uv);
  if (t.x == 1.0 && t.y == 1.0 && t.z == 1.0) {
    IntermediateTarget = v_FgColor;
  } else {
    IntermediateTarget = v_BgColor;
  }
}
