mod game;
mod graphics;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

pub const SQUARE_SIZE: u32 = 2;
pub const DEFAULT_PLAYGROUND_WIDTH: u32 = 100;
pub const DEFAULT_PLAYGROUND_HEIGHT: u32 = 100;
pub const DEFAULT_FRAME_RATE: u32 = 10;

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

  let mut game = game::GameOfLife::new(playground_height, playground_width);
  let mut graphics =
    graphics::Graphics::new(SQUARE_SIZE, playground_height, playground_width, frame_rate)
      .expect("failed to load graphics");

  let mut last_change: (u32, u32) = (0, 0);

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
                if *square == crate::game::Concept::Soil {
                  *square = crate::game::Concept::Sunflower;
                } else {
                  *square = crate::game::Concept::Soil;
                }
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
                    if *square == crate::game::Concept::Soil {
                      *square = crate::game::Concept::Sunflower;
                    } else {
                      *square = crate::game::Concept::Soil;
                    }
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
