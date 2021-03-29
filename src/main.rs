mod ashtest;
mod game;
mod graphics;

use crate::game::Concept;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

pub const SQUARE_SIZE: u32 = 8;
pub const DEFAULT_PLAYGROUND_WIDTH: u32 = 100;
pub const DEFAULT_PLAYGROUND_HEIGHT: u32 = 100;
pub const DEFAULT_FRAME_RATE: u32 = 2;

pub fn main() -> Result<(), String> {
  let args: Vec<String> = std::env::args().collect();
  let playground_height = if args.len() > 1 {
    str::parse::<u32>(&args[1]).expect("First argument must be integer height")
  } else {
    DEFAULT_PLAYGROUND_HEIGHT
  };
  let playground_width = if args.len() > 2 {
    str::parse::<u32>(&args[2]).expect("Second argument must be integer width")
  } else {
    DEFAULT_PLAYGROUND_WIDTH
  };
  let frame_rate = if args.len() > 3 {
    str::parse::<u32>(&args[3])
      .expect("Third argument must be integer frame rate, in frames-per-second")
  } else {
    DEFAULT_FRAME_RATE
  };
  let show_fps = if args.len() > 4 {
    str::parse::<u32>(&args[4]).expect("Third argument must be 1 or 0, to show or hide frame rate")
  } else {
    0
  };

  let mut game = game::GameOfLife::new(playground_height, playground_width);
  let mut graphics = graphics::Graphics::new(
    SQUARE_SIZE,
    playground_height,
    playground_width,
    frame_rate,
    show_fps == 1,
  )
  .expect("failed to load graphics");

  ashtest::run(&game.playground);

  let mut last_change: (u32, u32) = (0, 0);
  let mut curr_type = Concept::Sunflower;

  graphics
    .run(
      &mut game,
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
            game.toggle_state();
          }
          Event::MouseButtonDown {
            x,
            y,
            mouse_btn: MouseButton::Left,
            ..
          } => {
            let x = (x as u32) / SQUARE_SIZE;
            let y = (y as u32) / SQUARE_SIZE;
            match game.get_mut(x as i32, y as i32) {
              Some(square) => {
                if *square == Concept::Soil {
                  *square = Concept::Sunflower;
                } else if *square == Concept::Sunflower {
                  *square = Concept::Rose
                } else {
                  *square = Concept::Soil;
                }
                curr_type = *square;
                last_change = (x, y);
              }
              None => unreachable!(),
            };
          }
          Event::MouseMotion {
            x, y, mousestate, ..
          } => {
            if mousestate.left() {
              let x = (x as u32) / SQUARE_SIZE;
              let y = (y as u32) / SQUARE_SIZE;
              if (x, y) != last_change {
                last_change = (x, y);
                match game.get_mut(x as i32, y as i32) {
                  Some(square) => {
                    *square = curr_type;
                  }
                  None => unreachable!(),
                };
              }
            }
          }
          _ => {}
        }
        return false;
      },
      |game| {
        // update the game loop here
        if let game::State::Playing = game.state() {
          game.update();
        };
      },
    )
    .expect("Error in main loop");

  Ok(())
}
