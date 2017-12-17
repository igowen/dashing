#version 150 core

in vec2 a_Pos;
in vec2 a_Uv;

in vec2 a_Translate;
in vec4 a_FgColor;
in vec4 a_BgColor;
in uint a_Character;

uniform Locals {
  vec2 u_ScreenCharDim;
  vec2 u_FontCharDim;
};

out vec2 v_Uv;
out vec4 v_FgColor;
out vec4 v_BgColor;

void main() {
    vec2 font_offset = vec2(mod(a_Character, u_FontCharDim.x), floor(a_Character / u_FontCharDim.x));
    v_Uv = a_Uv / u_FontCharDim + font_offset / u_FontCharDim;
    v_FgColor = a_FgColor;
    v_BgColor = a_BgColor;
    gl_Position = vec4(a_Pos * 2.0 / u_ScreenCharDim + a_Translate, 0.0, 1.0);
}
