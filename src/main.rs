mod game;
mod graphics;

use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::keyboard::Keycode;

pub const SQUARE_SIZE: u32 = 2;
pub const PLAYGROUND_WIDTH: u32 = 500;
pub const PLAYGROUND_HEIGHT: u32 = 500;

pub fn main() -> Result<(), String> {  
  let mut game = game::GameOfLife::new(PLAYGROUND_HEIGHT, PLAYGROUND_WIDTH);
  let mut graphics = graphics::Graphics::new(SQUARE_SIZE, PLAYGROUND_HEIGHT, PLAYGROUND_WIDTH).expect("failed to load graphics");

  let mut frame : u32 = 0;
  let mut last_change : (u32,u32) = (0,0);

  graphics.run(&mut game, |game, event: Event|
    {
      match event {
        Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
            return true;
        },
        Event::KeyDown { keycode: Some(Keycode::Space), repeat: false, .. } => {
            game.toggle_state();
        },
        Event::MouseButtonDown { x, y, mouse_btn: MouseButton::Left, .. } => {
            let x = (x as u32) / SQUARE_SIZE;
            let y = (y as u32) / SQUARE_SIZE;
            match game.get_mut(x as i32, y as i32) {
                Some(square) => {*square = !(*square);},
                None => unreachable!(),
            };
        },
        Event::MouseMotion { x, y, mousestate, .. } => {
          if mousestate.left() {
            let x = (x as u32) / SQUARE_SIZE;
            let y = (y as u32) / SQUARE_SIZE;
            if (x,y) != last_change {
              last_change = (x,y);
              match game.get_mut(x as i32, y as i32) {
                  Some(square) => {*square = !(*square);}
                  None => unreachable!(),
              };
            }
          }
        },
        _ => {}
      } 
      return false;
    }, |game| {
      // update the game loop here
      if frame >= 1 {
        game.update();
        frame = 0;
      }

      if let game::State::Playing = game.state() {
        frame += 1;
      };
    }).expect("Error in main loop");

    Ok(())
}