//! `dashing` is a library for building roguelike games.
//!
//! # Example
//!
//! ```no_run
//! extern crate dashing;
//! extern crate sdl2;
//! // Screen dimensions in characters.
//! use dashing::*;
//! const WIDTH: u32 = 21;
//! const HEIGHT: u32 = 3;
//! pub fn main() {
//!     let pixels: Vec<u8> = vec![]; // Add your own texture here
//!     let tex = resources::sprite::SpriteTexture::new_from_pixels(
//!         &pixels[..],
//!         128,
//!         128,
//!         8,
//!         8,
//!         256).unwrap();
//!     let mut window = engine::window::WindowBuilder::new("dashing", WIDTH, HEIGHT, &tex)
//!         .build()
//!         .unwrap();
//!     let message = String::from("Swash your buckles!");
//!     'main: loop {
//!         let mut s = vec![engine::renderer::SpriteCellMeta::default(); 21 * 3];
//!         for (i, c) in message.chars().enumerate() {
//!             s[22 + i] = engine::renderer::SpriteCellMeta {
//!                 palette: resources::sprite::Palette::mono([0,0,0]).set(1, [255, 255, 255]),
//!                 sprite: c as u32,
//!             };
//!         }
//!         window.renderer_mut().update(s.iter());
//!         for event in window.render().unwrap() {
//!             match event {
//!                 sdl2::event::Event::Quit { .. } => {
//!                     break 'main;
//!                 }
//!                 _ => {}
//!             }
//!         }
//!     }
//! }
//!
//! ```
//!
//! # Roadmap
//! ## Features to be implemented
//!
//! * Input handling
//!   * Don't use SDL event types in the public interface
//! * GUI library
//!   * Splash screen support
//! * Entity-Component system
//! * Serialization/persistence framework
//! * Graphics improvements
//!   * ~~Palettized sprites~~
//!   * User-specified shaders
//!   * Animated sprites
//! * Resource management system
//!   * Build sprite map textures at runtime
//! * Audio
//! * Parallelism

#![deny(warnings)]
#![deny(missing_docs)]
#![allow(dead_code)]

#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_window_sdl;
extern crate gl;
extern crate itertools;
#[allow(unused)]
#[macro_use]
extern crate log;
extern crate sdl2;
extern crate time;

/// The `engine` module contains the interface for doing graphics, input, and sound. It has minimal
/// dependencies on other parts of the library, so if you just want an OpenGL-based sprite grid,
/// you could theoretically use it by itself without using the higher-level functionality provided
/// by other modules.
pub mod engine;

/// Functionality for loading and managing game data, such as sprite textures.
pub mod resources;

/// Functionality for building in-game UIs.
pub mod ui;

// Libraries used in tests.
#[cfg(test)]
extern crate euclid;
#[cfg(test)]
extern crate gleam;
#[cfg(test)]
extern crate image;
#[cfg(test)]
#[macro_use]
extern crate lazy_static;
#[cfg(test)]
extern crate offscreen_gl_context;
#[cfg(test)]
extern crate png;
#[cfg(test)]
extern crate pretty_logger;
#[cfg(test)]
#[macro_use]
extern crate hamcrest;
