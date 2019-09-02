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

uniform ScreenGlobals {
  vec2 u_ScreenSizeInPixels;
  uint u_FrameCounter;
  float u_ElapsedTime;
  vec2 u_ScaleFactor;
};

in vec2 a_Pos;
in vec2 a_Uv;

out vec2 v_Uv;

void main() {
  v_Uv = a_Uv;
  gl_Position = vec4(a_Pos * u_ScaleFactor, 0.0, 1.0);
}
