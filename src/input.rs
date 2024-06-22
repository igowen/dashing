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

// TODO: implement a real event system instead of exposing the raw winit events.
pub use winit::event::WindowEvent;

/// Event type for sprite-level mouse movement.
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct MouseMoved {
    /// Sprite position
    pub sprite_position: (u32, u32),
    /// Pixel position
    pub absolute_position: (f64, f64),
}
