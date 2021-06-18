#![feature(async_stream)]
#[macro_use]
extern crate rocket;

mod args;
mod game;
mod graphics;

use crate::args::parse_args;
use crate::game::CellState;
use crate::game::GameState;
use rocket::fs::NamedFile;
use rocket::State;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
  fn info(&self) -> Info {
    Info {
      name: "Add CORS headers to responses",
      kind: Kind::Response,
    }
  }

  async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
    response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
    response.set_header(Header::new(
      "Access-Control-Allow-Methods",
      "POST, GET, PATCH, OPTIONS",
    ));
    response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
    response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
  }
}

#[get("/")]
async fn index() -> Option<NamedFile> {
  NamedFile::open("target/index.html").await.ok()
}

#[get("/garden")]
fn garden(state: &State<Arc<GameState>>) -> &[u8] {
  let game = state.inner();

  unsafe {
    return std::slice::from_raw_parts(
      game.game_data.load(Relaxed) as *const u8,
      game.game_size.load(Relaxed) as usize
        * (std::mem::size_of::<CellState>() / std::mem::size_of::<u8>()),
    );
  };
}

#[launch]
fn rocket() -> _ {
  let args = parse_args(std::env::args().skip(1).collect());

  let (snd_state, rcv_state) = std::sync::mpsc::channel::<Arc<GameState>>();

  std::thread::spawn(move || {
    let mut runner = game::Runner::new(args.size);
    snd_state.send(runner.game_state.clone()).unwrap();

    let clock = SystemTime::now();
    let mut next_frame: u128 = clock.elapsed().unwrap().as_millis();

    loop {
      let curr_time = clock.elapsed().unwrap().as_millis();
      if curr_time >= next_frame {
        next_frame = curr_time + (1000 / (args.update_rate as u128));
        runner.execute();
      } else {
        std::thread::sleep(Duration::from_millis((next_frame - curr_time) as u64));
      }
    }
  });

  let data = rcv_state.recv().unwrap();
  let data2 = data.clone();
  if args.show_graphics {
    std::thread::spawn(move || {
      let mut graphics =
        graphics::Graphics::new(args.pixel_size, args.size, args.draw_rate, args.show_fps)
          .expect("failed to load graphics");

      graphics
        .run(data, |event: Event| {
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
              // game.toggle_pause();
            }
            Event::MouseButtonDown {
              x,
              y,
              mouse_btn: MouseButton::Left,
              ..
            } => {
              let x = (x as u32) / args.pixel_size;
              let y = (y as u32) / args.pixel_size;
              /*game.mutate(Mutation {
                x,
                y,
                concept: Concept::Rose,
              })*/
            }
            Event::MouseMotion {
              x, y, mousestate, ..
            } => {
              if mousestate.left() {
                let x = (x as u32) / args.pixel_size;
                let y = (y as u32) / args.pixel_size;
                /*game.mutate(Mutation {
                  x,
                  y,
                  concept: Concept::Rose,
                })*/
              }
            }
            _ => {}
          }
          return false;
        })
        .expect("Error in main loop");
    });
  }

  rocket::build()
    .manage(data2)
    .mount("/", routes![index, garden])
    .attach(CORS)
}
