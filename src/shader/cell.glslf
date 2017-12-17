#version 150 core

uniform sampler2D t_Texture;
uniform Locals {
  vec2 u_ScreenCharDim;
  vec2 u_FontCharDim;
};

in vec2 v_Uv;
in vec4 v_FgColor;
in vec4 v_BgColor;

out vec4 Target0;

void main() {
    vec4 t = texture(t_Texture, v_Uv);
    if (t.x == 1.0 && t.y == 1.0 && t.z == 1.0) {
      Target0 = v_FgColor;
    } else {
      Target0 = v_BgColor;
    }
}
