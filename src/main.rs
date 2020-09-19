extern crate sdl2;

use sdl2::rect::{Point, Rect};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::keyboard::Keycode;
use sdl2::video::{Window, WindowContext};
use sdl2::render::{Canvas, Texture, TextureCreator};
use crate::game_of_life::{SQUARE_SIZE, PLAYGROUND_WIDTH, PLAYGROUND_HEIGHT};

mod game_of_life {
    pub const SQUARE_SIZE: u32 = 2;
    pub const PLAYGROUND_WIDTH: u32 = 500;
    pub const PLAYGROUND_HEIGHT: u32 = 500;

    #[derive(Copy, Clone)]
    pub enum State {
        Paused,
        Playing,
    }

    pub struct GameOfLife {
        playground: [bool; (PLAYGROUND_WIDTH*PLAYGROUND_HEIGHT) as usize],
        state: State,
    }

    impl GameOfLife {
        pub fn new() -> GameOfLife {
            let mut playground = [false; (PLAYGROUND_WIDTH * PLAYGROUND_HEIGHT) as usize];

            // let's make a nice default pattern !
            for i in 1..(PLAYGROUND_HEIGHT-1) {
                playground[(1 + i* PLAYGROUND_WIDTH) as usize] = true;
                playground[((PLAYGROUND_WIDTH-2) + i* PLAYGROUND_WIDTH) as usize] = true;
            }
            for j in 2..(PLAYGROUND_WIDTH-2) {
                playground[(PLAYGROUND_WIDTH + j) as usize] = true;
                playground[((PLAYGROUND_HEIGHT-2)*PLAYGROUND_WIDTH + j) as usize] = true;
            }

            GameOfLife {
                playground: playground,
                state: State::Paused,
            }
        }

        pub fn get(&self, x: i32, y: i32) -> Option<bool> {
            if x >= 0 && y >= 0 &&
               (x as u32) < PLAYGROUND_WIDTH && (y as u32) < PLAYGROUND_HEIGHT {
                Some(self.playground[(x as u32 + (y as u32)* PLAYGROUND_WIDTH) as usize])
            } else {
                None
            }
        }

        pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut bool> {
            if x >= 0 && y >= 0 &&
               (x as u32) < PLAYGROUND_WIDTH && (y as u32) < PLAYGROUND_HEIGHT {
                Some(&mut self.playground[(x as u32 + (y as u32)* PLAYGROUND_WIDTH) as usize])
            } else {
                None
            }
        }

        pub fn toggle_state(&mut self) {
            self.state = match self.state {
                State::Paused => State::Playing,
                State::Playing => State::Paused,
            }
        }

        pub fn state(&self) -> State {
            self.state
        }

        pub fn update(&mut self) {
            let mut new_playground = self.playground;
            for (u, square) in new_playground.iter_mut().enumerate() {
                let u = u as u32;
                let x = u % PLAYGROUND_WIDTH;
                let y = u / PLAYGROUND_WIDTH;
                let mut count : u32 = 0;
                for i in -1..2 {
                    for j in -1..2 {
                        if !(i == 0 && j == 0) {
                            let peek_x : i32 = (x as i32) + i;
                            let peek_y : i32 = (y as i32) + j;
                            if let Some(true) = self.get(peek_x, peek_y) {
                                count += 1;
                            }
                        }
                    }
                }
                if count > 3 || count < 2 {
                    *square = false;
                } else if count == 3 {
                    *square = true;
                } else if count == 2 {
                    *square = *square;
                }
            }
            self.playground = new_playground;
        }
    }



    impl<'a> IntoIterator for &'a GameOfLife {
        type Item = &'a bool;
        type IntoIter = ::std::slice::Iter<'a, bool>;
        fn into_iter(self) -> ::std::slice::Iter<'a, bool> {
            self.playground.iter()
        }
    }
}

fn dummy_texture<'a>(canvas: &mut Canvas<Window>, texture_creator: &'a TextureCreator<WindowContext>) -> Result<Texture<'a>, String> {
    let mut square_texture = texture_creator.create_texture_target(None, SQUARE_SIZE, SQUARE_SIZE).map_err(|e| e.to_string())?;

    // let's change the textures we just created
    canvas.with_texture_canvas(&mut square_texture, |texture_canvas| {
        texture_canvas.set_draw_color(Color::RGB(0, 0, 0));
        texture_canvas.clear();
        texture_canvas.set_draw_color(Color::RGB(192, 192, 192));
        texture_canvas.fill_rect(Rect::new(0,0,SQUARE_SIZE, SQUARE_SIZE))
            .expect("could not draw point");
    }).map_err(|e| e.to_string())?;
    
    Ok(square_texture)
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // the window is the representation of a window in your operating system,
    // however you can only manipulate properties of that window, like its size, whether it's
    // fullscreen, ... but you cannot change its content without using a Canvas or using the
    // `surface()` method.
    let window = video_subsystem
        .window("rust-sdl2 demo: Game of Life",
                SQUARE_SIZE*PLAYGROUND_WIDTH,
                SQUARE_SIZE*PLAYGROUND_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    // the canvas allows us to both manipulate the property of the window and to change its content
    // via hardware or software rendering. See CanvasBuilder for more info.
    let mut canvas = window.into_canvas()
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
    let texture_creator : TextureCreator<_> = canvas.texture_creator();

    // Create a "target" texture so that we can use our Renderer with it later
    let square_texture = dummy_texture(&mut canvas, &texture_creator)?;
    let mut game = game_of_life::GameOfLife::new();

    let mut event_pump = sdl_context.event_pump()?;
    let mut frame : u32 = 0;
    let mut last_change : (u32,u32) = (0,0);
    'running: loop {
        // get the inputs here
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
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
        }

        // update the game loop here
        if frame >= 1 {
            game.update();
            frame = 0;
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for (i, unit) in (&game).into_iter().enumerate() {
            let i = i as u32;
            if *unit {
                canvas.copy(&square_texture,
                            None,
                            Rect::new(((i % PLAYGROUND_WIDTH) * SQUARE_SIZE) as i32,
                                      ((i / PLAYGROUND_WIDTH) * SQUARE_SIZE) as i32,
                                      SQUARE_SIZE,
                                      SQUARE_SIZE))?;
            }
        }
        canvas.present();
        if let game_of_life::State::Playing = game.state() {
            frame += 1;
        };
    }

    Ok(())
}