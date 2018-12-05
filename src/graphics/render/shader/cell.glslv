// Copyright 2018 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
