#[macro_use]
extern crate rocket;

mod game;
mod graphics;

use crate::game::Concept;
use crate::game::Mutation;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

pub const SQUARE_SIZE: u32 = 8;
pub const DEFAULT_PLAYGROUND_WIDTH: u32 = 100;
pub const DEFAULT_FRAME_RATE: u32 = 2;

#[get("/")]
fn index() -> &'static str {
  "Hello, world!"
}

#[launch]
fn rocket() -> _ {
  let args: Vec<String> = std::env::args().collect();
  let playground_width = if args.len() > 1 {
    str::parse::<u32>(&args[1]).expect("First argument must be integer width")
  } else {
    DEFAULT_PLAYGROUND_WIDTH
  };
  let frame_rate = if args.len() > 2 {
    str::parse::<u32>(&args[2])
      .expect("Second argument must be integer frame rate, in frames-per-second")
  } else {
    DEFAULT_FRAME_RATE
  };
  let show_fps = if args.len() > 3 {
    str::parse::<u32>(&args[3]).expect("Third argument must be 1 or 0, to show or hide frame rate")
  } else {
    0
  };

  std::thread::spawn(move || {
    let mut graphics =
      graphics::Graphics::new(SQUARE_SIZE, playground_width, frame_rate, show_fps == 1)
        .expect("failed to load graphics");

    let mut runner = game::Runner::new(playground_width);

    graphics
      .run(
        &mut runner,
        |game, event: Event| {
          match event {
            Event::Quit { .. }
            | Event::KeyDown {
              keycode: Some(Keycode::Escape),
              ..
            } => {
              return true;
            }
            Event::KeyDown {
              keycode: Some(Keycode::Space),
              repeat: false,
              ..
            } => {
              game.toggle_pause();
            }
            Event::MouseButtonDown {
              x,
              y,
              mouse_btn: MouseButton::Left,
              ..
            } => {
              let x = (x as u32) / SQUARE_SIZE;
              let y = (y as u32) / SQUARE_SIZE;
              game.mutate(Mutation {
                x,
                y,
                concept: Concept::Rose,
              })
            }
            Event::MouseMotion {
              x, y, mousestate, ..
            } => {
              if mousestate.left() {
                let x = (x as u32) / SQUARE_SIZE;
                let y = (y as u32) / SQUARE_SIZE;
                game.mutate(Mutation {
                  x,
                  y,
                  concept: Concept::Rose,
                })
              }
            }
            _ => {}
          }
          return false;
        },
        |runner| {
          runner.execute();
        },
      )
      .expect("Error in main loop");
  });

  rocket::build().mount("/", routes![index])
}
