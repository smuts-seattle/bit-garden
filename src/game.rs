#[derive(Copy, Clone)]
pub enum State {
  Paused,
  Playing,
}

pub struct GameOfLife {
  playground: Vec<bool>,
  state: State,
  world_height: u32,
  world_width: u32
}

impl GameOfLife {
  pub fn new(world_height: u32, world_width: u32) -> GameOfLife {
    let mut playground = vec![false; (world_width * world_height) as usize];

    // let's make a nice default pattern !
    for i in 1..(world_height-1) {
      playground[(1 + i* world_width) as usize] = true;
      playground[((world_width-2) + i* world_width) as usize] = true;
    }
    for j in 2..(world_width-2) {
      playground[(world_width + j) as usize] = true;
      playground[((world_height-2)*world_width + j) as usize] = true;
    }

    GameOfLife {
      playground: playground,
      state: State::Paused,
      world_height: world_height,
      world_width: world_width
    }
  }

  pub fn get(&self, x: i32, y: i32) -> Option<bool> {
    if x >= 0 && y >= 0 &&
      (x as u32) < self.world_width && (y as u32) < self.world_height {
      Some(self.playground[(x as u32 + (y as u32)* self.world_width) as usize])
    } else {
      None
    }
  }

  pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut bool> {
    if x >= 0 && y >= 0 &&
      (x as u32) < self.world_width && (y as u32) < self.world_height {
      Some(&mut self.playground[(x as u32 + (y as u32)* self.world_width) as usize])
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
    let mut new_playground = self.playground.to_vec();
    for (u, square) in new_playground.iter_mut().enumerate() {
      let u = u as u32;
      let x = u % self.world_width;
      let y = u / self.world_width;
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