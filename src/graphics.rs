extern crate sdl2;

use crate::game::Concept;
use crate::game::Runner;
use crate::CellState;
use core::sync::atomic::AtomicPtr;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use std::path::Path;
use std::sync::Arc;

const SOIL_COLOR: Color = Color::RGB(53, 48, 40);
const SUNFLOWER_COLOR: Color = Color::RGB(100, 87, 39);
const ROSE_COLOR: Color = Color::RGB(191, 67, 66);
const DOGWOOD_COLOR: Color = Color::RGB(234, 213, 230);
const DOGWOOD_COLOR_FADED: Color = Color::RGB(185, 168, 178);
const DOGWOOD_COLOR_RELIC: Color = Color::RGB(119, 108, 109);
const DOGWOOD_COLOR_RUIN: Color = Color::RGB(69, 63, 57);

const FPS_COLOR: Color = Color::RGB(208, 240, 192);

pub struct Graphics {
  sdl_context: sdl2::Sdl,
  canvas: Canvas<Window>,
  texture_creator: TextureCreator<WindowContext>,
  square_size: u32,
  world_width: u32,
  frame_rate: u32,
  show_fps: bool,
}

pub struct Textures<'a> {
  sunflower: Texture<'a>,
  rose: Texture<'a>,
  dogwood: Texture<'a>,
  dogwood_faded: Texture<'a>,
  dogwood_relic: Texture<'a>,
  dogwood_ruin: Texture<'a>,
}

impl Graphics {
  pub fn new(
    square_size: u32,
    world_width: u32,
    frame_rate: u32,
    show_fps: bool,
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
        square_size * world_width,
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
      frame_rate,
      show_fps,
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
    let mut rose_texture = texture_creator
      .create_texture_target(None, square_size, square_size)
      .map_err(|e| e.to_string())?;
    let mut dogwood_texture = texture_creator
      .create_texture_target(None, square_size, square_size)
      .map_err(|e| e.to_string())?;
    let mut dogwood_faded_texture = texture_creator
      .create_texture_target(None, square_size, square_size)
      .map_err(|e| e.to_string())?;
    let mut dogwood_relic_texture = texture_creator
      .create_texture_target(None, square_size, square_size)
      .map_err(|e| e.to_string())?;
    let mut dogwood_ruin_texture = texture_creator
      .create_texture_target(None, square_size, square_size)
      .map_err(|e| e.to_string())?;

    let textures = vec![
      (&mut sunflower_texture, 1),
      (&mut rose_texture, 2),
      (&mut dogwood_texture, 3),
      (&mut dogwood_faded_texture, 4),
      (&mut dogwood_relic_texture, 5),
      (&mut dogwood_ruin_texture, 6),
    ];

    let square_size = square_size;
    // let's change the textures we just created
    canvas
      .with_multiple_texture_canvas(textures.iter(), |texture_canvas, i| {
        texture_canvas.set_draw_color(SOIL_COLOR);
        texture_canvas.clear();
        match *i {
          1 => {
            texture_canvas.set_draw_color(SUNFLOWER_COLOR);
            texture_canvas
              .fill_rect(Rect::new(0, 0, square_size, square_size))
              .expect("could not draw point");
          }
          2 => {
            texture_canvas.set_draw_color(ROSE_COLOR);
            texture_canvas
              .fill_rect(Rect::new(0, 0, square_size, square_size))
              .expect("could not draw point");
          }
          3 => {
            texture_canvas.set_draw_color(DOGWOOD_COLOR);
            texture_canvas
              .fill_rect(Rect::new(0, 0, square_size, square_size))
              .expect("could not draw point");
          }
          4 => {
            texture_canvas.set_draw_color(DOGWOOD_COLOR_FADED);
            texture_canvas
              .fill_rect(Rect::new(0, 0, square_size, square_size))
              .expect("could not draw point");
          }
          5 => {
            texture_canvas.set_draw_color(DOGWOOD_COLOR_RELIC);
            texture_canvas
              .fill_rect(Rect::new(0, 0, square_size, square_size))
              .expect("could not draw point");
          }
          6 => {
            texture_canvas.set_draw_color(DOGWOOD_COLOR_RUIN);
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
      rose: rose_texture,
      dogwood: dogwood_texture,
      dogwood_faded: dogwood_faded_texture,
      dogwood_relic: dogwood_relic_texture,
      dogwood_ruin: dogwood_ruin_texture,
    })
  }

  pub fn run<FEvent>(
    &mut self,
    state: Arc<CellState>,
    state_size: u32,
    mut on_event: FEvent,
  ) -> Result<(), String>
  where
    FEvent: FnMut(Event) -> bool,
  {
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let mut timer = self.sdl_context.timer()?;
    let mut last_frame = 0;
    let mut frame_rate = 0;
    let mut event_pump = self.sdl_context.event_pump()?;

    // Load a font
    let font = ttf_context.load_font(Path::new("fonts/NotoMono-Regular.ttf"), 14)?;

    let textures =
      Graphics::dummy_texture(&mut self.canvas, &self.texture_creator, self.square_size)
        .expect("could not build square texture");
    let raw_state = Arc::into_raw(state);
    'exit: loop {
      // get the inputs here
      for event in event_pump.poll_iter() {
        if on_event(event) {
          break 'exit Ok(());
        }
      }

      self.canvas.set_draw_color(SOIL_COLOR);
      self.canvas.clear();

      for i in 0..state_size {
        let i = i as u32;
        unsafe {
          let unit = *raw_state.offset(i as isize);

          match unit.concept {
            Concept::Sunflower => {
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
            Concept::Rose => {
              self.canvas.copy(
                &textures.rose,
                None,
                Rect::new(
                  ((i % self.world_width) * self.square_size) as i32,
                  ((i / self.world_width) * self.square_size) as i32,
                  self.square_size,
                  self.square_size,
                ),
              )?;
            }
            Concept::Dogwood => {
              if unit.blood < -35 {
                self.canvas.copy(
                  &textures.dogwood,
                  None,
                  Rect::new(
                    ((i % self.world_width) * self.square_size) as i32,
                    ((i / self.world_width) * self.square_size) as i32,
                    self.square_size,
                    self.square_size,
                  ),
                )?;
              } else if unit.blood < -20 {
                self.canvas.copy(
                  &textures.dogwood_faded,
                  None,
                  Rect::new(
                    ((i % self.world_width) * self.square_size) as i32,
                    ((i / self.world_width) * self.square_size) as i32,
                    self.square_size,
                    self.square_size,
                  ),
                )?;
              } else if unit.blood < -5 {
                self.canvas.copy(
                  &textures.dogwood_relic,
                  None,
                  Rect::new(
                    ((i % self.world_width) * self.square_size) as i32,
                    ((i / self.world_width) * self.square_size) as i32,
                    self.square_size,
                    self.square_size,
                  ),
                )?;
              } else {
                self.canvas.copy(
                  &textures.dogwood_ruin,
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
            _ => {}
          }
        }
      }
      if self.show_fps {
        let surface = font
          .render(&format!("{:03}", frame_rate))
          .blended(FPS_COLOR)
          .map_err(|e| e.to_string())?;
        let texture = self
          .texture_creator
          .create_texture_from_surface(&surface)
          .map_err(|e| e.to_string())?;
        self
          .canvas
          .copy(&texture, None, Some(Rect::new(0, 0, 42, 14)))?;
      }
      self.canvas.present();
    }
  }
}
