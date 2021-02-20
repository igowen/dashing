use dashing::*;
use pretty_logger;
use time;

// Screen dimensions in characters.
const WIDTH: u32 = 21;
const HEIGHT: u32 = 3;

// Client code must implement a "driver", which is a combination of input handler and
// interface to the renderer.
struct ExampleDriver {
    // SpriteLayer is a convenient abstraction for representing an orthogonal region of the
    // screen.
    root_layer: graphics::drawing::SpriteLayer,
    message: String,
}

impl Driver for ExampleDriver {
    // The driver handles all the user input events the window receives. The `handle_input`
    // method needs to be lightweight, because it blocks the render thread.
    fn handle_input(&mut self, e: Event) -> EngineSignal {
        match e {
            Event::WindowCloseRequested | Event::WindowDestroyed => EngineSignal::Halt,
            _ => EngineSignal::Continue,
        }
    }

    // After the engine processes all pending events, `process_frame` is called. This is where
    // you would update the screen, as well as run any per-frame processing your game
    // requires.
    fn process_frame<R>(&mut self, renderer: &mut R) -> EngineSignal
    where
        R: dashing::graphics::render::RenderInterface,
    {
        // Clear the sprite layer.
        self.root_layer.clear();
        // Print the message to the screen.
        for (i, c) in self.message.chars().enumerate() {
            self.root_layer[WIDTH as usize + 1 + i] = graphics::drawing::SpriteCell {
                palette: resources::color::Palette::mono([0, 0, 0]).set(
                    1,
                    resources::color::Color::from_hsv(
                        time::OffsetDateTime::now_utc().millisecond() as f32 * 360.0 / 1000.0,
                        1.0 - (i as f32 / self.message.len() as f32) * 0.5,
                        1.0,
                    ),
                ),
                sprite: c as u32,
                transparent: false,
            };
        }
        // Update the renderer.
        renderer.update(self.root_layer.iter());

        EngineSignal::Continue
    }
}

pub fn main() {
    pretty_logger::init_to_defaults().unwrap();

    let tex_png = include_bytes!("../src/graphics/render/testdata/12x12.png");
    let mut decoder = png::Decoder::new(&tex_png[..]);
    decoder.set_transformations(png::Transformations::IDENTITY);

    let (info, mut reader) = decoder.read_info().unwrap();
    assert!(info.color_type == png::ColorType::Indexed);
    let mut imgdata = vec![0u8; reader.output_buffer_size()];
    reader.next_frame(&mut imgdata[..]).unwrap();
    let tex = dashing::resources::sprite::SpriteTexture::new_from_pixels(
        &imgdata[..],
        reader.info().size().0 as usize,
        reader.info().size().1 as usize,
        reader.info().size().0 as usize / 16,
        reader.info().size().1 as usize / 16,
        256,
    )
    .unwrap();

    let window_builder = window::WindowBuilder::new("hello world", WIDTH, HEIGHT, &tex)
        .with_clear_color((0.2, 0.2, 0.2).into())
        .with_vsync(true);

    let driver = ExampleDriver {
        root_layer: graphics::drawing::SpriteLayer::new(WIDTH as usize, HEIGHT as usize),
        message: String::from("Swash your buckles!"),
    };

    let engine = dashing::Engine::new(window_builder, driver).unwrap();

    engine.run();
}
