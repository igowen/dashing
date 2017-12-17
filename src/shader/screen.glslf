#version 150 core

uniform sampler2D t_ScreenTexture;

uniform Locals {
  int u_FrameCounter;
};

in vec2 v_Uv;
out vec4 Target0;

void main() {
  vec4 t = texture(t_ScreenTexture, v_Uv);
  Target0 = t;
}
