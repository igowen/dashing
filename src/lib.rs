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
//!     let mut window = window::WindowBuilder::new("dashing", WIDTH, HEIGHT, &tex)
//!         .build()
//!         .unwrap();
//!     let message = String::from("Swash your buckles!");
//!     'main: loop {
//!         let mut s = vec![graphics::SpriteCell::default(); 21 * 3];
//!         for (i, c) in message.chars().enumerate() {
//!             s[22 + i] = graphics::SpriteCell {
//!                 palette: resources::color::Palette::mono([0,0,0]).set(1, [255, 255, 255]),
//!                 sprite: c as u32,
//!                 transparent: false,
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
//!   * User-specified shaders
//!   * Animated sprites
//! * Resource management system
//!   * Build sprite map textures at runtime
//! * Audio
//! * Parallelism

#![deny(missing_docs)]
#![allow(dead_code)]
#![feature(trait_alias)]

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

/// Routines for creating and managing the game window.
pub mod window;

/// API for interacting with the low-level rendering system.
pub mod graphics;

/// Functionality for loading and managing game data, such as sprite textures.
pub mod resources;

/// Functionality for building in-game UIs.
pub mod ui;
