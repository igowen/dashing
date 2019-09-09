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

uniform usampler2D t_SpriteTexture;
uniform sampler2D t_Palette;

uniform CellGlobals {
  uvec2 u_ScreenSizeInSprites;
  uvec2 u_SpriteMapDimensions;
};

in vec2 v_Uv;
flat in uint v_Index;
in vec2 v_SpritePos;

out vec4 IntermediateTarget;

void main() {
  vec4 t = texture(t_SpriteTexture, v_Uv);
  vec4 p = texelFetch(t_Palette,
                      ivec2(v_SpritePos.x * 16 + clamp(t.x, 0, 15),
                            v_SpritePos.y),
                      0);
  IntermediateTarget = p;
}
