extern crate sdl2;

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use std::path::Path;

use crate::game::Concept;
use crate::game::GameOfLife;

const SOIL_COLOR: Color = Color::RGB(53, 48, 40);
const SUNFLOWER_COLOR: Color = Color::RGB(100, 87, 39);

pub struct Graphics {
  sdl_context: sdl2::Sdl,
  canvas: Canvas<Window>,
  texture_creator: TextureCreator<WindowContext>,
  square_size: u32,
  world_width: u32,
  world_height: u32,
  frame_rate: u32,
}

pub struct Textures<'a> {
  sunflower: Texture<'a>,
  red: Texture<'a>,
  yellow: Texture<'a>,
}

impl Graphics {
  pub fn new(
    square_size: u32,
    world_height: u32,
    world_width: u32,
    frame_rate: u32,
  ) -> Result<Graphics, String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // the window is the representation of a window in your operating system,
    // however you can only manipulate properties of that window, like its size, whether it's
    // fullscreen, ... but you cannot change its content without using a Canvas or using the
    // `surface()` method.
    let window = video_subsystem
      .window(
        "bit-garden",
        square_size * world_width,
        square_size * world_height,
      )
      .position_centered()
      .build()
      .map_err(|e| e.to_string())?;

    // the canvas allows us to both manipulate the property of the window and to change its content
    // via hardware or software rendering. See CanvasBuilder for more info.
    let mut canvas = window
      .into_canvas()
      .target_texture()
      .present_vsync()
      .build()
      .map_err(|e| e.to_string())?;

    println!("Using SDL_Renderer \"{}\"", canvas.info().name);
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    // clears the canvas with the color we set in `set_draw_color`.
    canvas.clear();
    // However the canvas has not been updated to the window yet, everything has been processed to
    // an internal buffer, but if we want our buffer to be displayed on the window, we need to call
    // `present`. We need to call this everytime we want to render a new frame on the window.
    canvas.present();

    // this struct manages textures. For lifetime reasons, the canvas cannot directly create
    // textures, you have to create a `TextureCreator` instead.
    let texture_creator = canvas.texture_creator();

    return Ok(Graphics {
      sdl_context: sdl_context,
      canvas: canvas,
      texture_creator: texture_creator,
      square_size: square_size,
      world_width: world_width,
      world_height: world_height,
      frame_rate,
    });
  }

  fn dummy_texture<'a>(
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    square_size: u32,
  ) -> Result<Textures<'a>, String> {
    let mut sunflower_texture = texture_creator
      .create_texture_target(None, square_size, square_size)
      .map_err(|e| e.to_string())?;
    let mut red_texture = texture_creator
      .create_texture_target(None, square_size, square_size)
      .map_err(|e| e.to_string())?;
    let mut yellow_texture = texture_creator
      .create_texture_target(None, square_size, square_size)
      .map_err(|e| e.to_string())?;
    let textures = vec![
      (&mut sunflower_texture, 1),
      (&mut red_texture, 2),
      (&mut yellow_texture, 3),
    ];

    let square_size = square_size;
    // let's change the textures we just created
    canvas
      .with_multiple_texture_canvas(textures.iter(), |texture_canvas, i| {
        texture_canvas.set_draw_color(Color::RGB(0, 0, 0));
        texture_canvas.clear();
        match *i {
          1 => {
            texture_canvas.set_draw_color(SUNFLOWER_COLOR);
            texture_canvas
              .fill_rect(Rect::new(0, 0, square_size, square_size))
              .expect("could not draw point");
          }
          2 => {
            texture_canvas.set_draw_color(Color::RGB(192, 0, 0));
            texture_canvas
              .fill_rect(Rect::new(0, 0, square_size, square_size))
              .expect("could not draw point");
          }
          3 => {
            texture_canvas.set_draw_color(Color::RGB(192, 192, 0));
            texture_canvas
              .fill_rect(Rect::new(0, 0, square_size, square_size))
              .expect("could not draw point");
          }
          _ => {}
        }
      })
      .map_err(|e| e.to_string())?;
    Ok(Textures {
      sunflower: sunflower_texture,
      red: red_texture,
      yellow: yellow_texture,
    })
  }

  pub fn run<FEvent, FFrame>(
    &mut self,
    game: &mut GameOfLife,
    mut on_event: FEvent,
    mut on_frame: FFrame,
  ) -> Result<(), String>
  where
    FEvent: FnMut(&mut GameOfLife, Event) -> bool,
    FFrame: FnMut(&mut GameOfLife),
  {
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let mut timer = self.sdl_context.timer()?;
    let mut last_frame = 0;
    let mut frame_rate = 0;
    let mut event_pump = self.sdl_context.event_pump()?;

    // Load a font
    let mut font = ttf_context.load_font(Path::new("fonts/NotoMono-Regular.ttf"), 128)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    let textures =
      Graphics::dummy_texture(&mut self.canvas, &self.texture_creator, self.square_size)
        .expect("could not build square texture");

    'exit: loop {
      // render a surface, and convert it to a texture bound to the canvas
      let surface = font
        .render(&std::string::ToString::to_string(&frame_rate))
        .blended(Color::RGB(255, 0, 0))
        .map_err(|e| e.to_string())?;
      let texture = self
        .texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;
      // get the inputs here
      for event in event_pump.poll_iter() {
        if on_event(game, event) {
          break 'exit Ok(());
        }
      }

      let ticks = timer.ticks();
      if (ticks - last_frame) >= (1000 / self.frame_rate) {
        on_frame(game);
        frame_rate = 1000 / (ticks - last_frame);
        last_frame = ticks;
      }

      self.canvas.set_draw_color(SOIL_COLOR);
      self.canvas.clear();
      self
        .canvas
        .copy(&texture, None, Some(Rect::new(0, 0, 14, 14)))?;

      for (i, unit) in game.into_iter().enumerate() {
        let i = i as u32;
        if unit.concept == Concept::Sunflower {
          self.canvas.copy(
            &textures.sunflower,
            None,
            Rect::new(
              ((i % self.world_width) * self.square_size) as i32,
              ((i / self.world_width) * self.square_size) as i32,
              self.square_size,
              self.square_size,
            ),
          )?;
        }
      }
      self.canvas.present();
    }
  }
}
