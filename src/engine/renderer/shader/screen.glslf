#version 150 core

uniform sampler2D t_ScreenTexture;

uniform ScreenGlobals {
  vec2 u_ScreenSizeInPixels;
  int u_FrameCounter;
  float u_ElapsedTime;
};

in vec2 v_Uv;
out vec4 Target0;

void main()
{
  Target0 = texture(t_ScreenTexture, v_Uv);
}
