#[derive(Copy, Clone)]
pub enum State {
  Paused,
  Playing,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Concept {
  Soil = 0,
  Sunflower = 1,
  Rose = 2,
  Dogwood = 3,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CellState {
  pub concept: Concept,
  pub blood: i32,
}

pub struct GameOfLife {
  pub playground: Vec<CellState>,
  state: State,
  world_height: u32,
  world_width: u32,
}

fn to_dust(square: &mut CellState) {
  square.concept = Concept::Soil;
  square.blood = 0;
}

impl GameOfLife {
  pub fn new(world_height: u32, world_width: u32) -> GameOfLife {
    let mut playground = vec![
      CellState {
        concept: Concept::Soil,
        blood: 0
      };
      (world_width * world_height) as usize
    ];

    // let's make a nice default pattern
    for i in 1..(world_height - 1) {
      playground[(1 + i * world_width) as usize].concept = Concept::Sunflower;
      playground[((world_width - 2) + i * world_width) as usize].concept = Concept::Sunflower;
    }
    for j in 2..(world_width - 2) {
      playground[(world_width + j) as usize].concept = Concept::Sunflower;
      playground[((world_height - 2) * world_width + j) as usize].concept = Concept::Sunflower;
    }

    GameOfLife {
      playground: playground,
      state: State::Paused,
      world_height: world_height,
      world_width: world_width,
    }
  }

  pub fn get(&self, x: i32, y: i32) -> Option<CellState> {
    if x >= 0 && y >= 0 && (x as u32) < self.world_width && (y as u32) < self.world_height {
      Some(self.playground[(x as u32 + (y as u32) * self.world_width) as usize])
    } else {
      None
    }
  }

  pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut Concept> {
    if x >= 0 && y >= 0 && (x as u32) < self.world_width && (y as u32) < self.world_height {
      Some(&mut self.playground[(x as u32 + (y as u32) * self.world_width) as usize].concept)
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

  fn check_nearby<FCheck>(
    &self,
    x: u32,
    y: u32,
    radius_outer: i32,
    radius_inner: i32,
    mut check: FCheck,
  ) where
    FCheck: FnMut(&CellState) -> bool,
  {
    for i in 0 - radius_outer..1 + radius_outer {
      for j in 0 - radius_outer..1 + radius_outer {
        if !(i.abs() < radius_inner && j.abs() < radius_inner) {
          let peek_x: i32 = (x as i32) + i;
          let peek_y: i32 = (y as i32) + j;
          match self.get(peek_x, peek_y) {
            Some(cell) => {
              if check(&cell) {
                return;
              }
            }
            _ => {}
          }
        }
      }
    }
  }

  pub fn update(&mut self) {
    let mut new_playground = self.playground.to_vec();

    for (u, square) in new_playground.iter_mut().enumerate() {
      let u = u as u32;
      let x = u % self.world_width;
      let y = u / self.world_width;
      if square.concept == Concept::Soil || square.concept == Concept::Sunflower {
        let mut count: u32 = 0;
        self.check_nearby(x, y, 1, 1, |cell| {
          if cell.concept == Concept::Sunflower {
            count += 1;
          }
          return false;
        });
        if square.concept == Concept::Sunflower && (count > 3 || count < 2) {
          // Death
          let mut loved = false;
          self.check_nearby(x, y, 2, 2, |cell| {
            if cell.concept == Concept::Rose {
              loved = true;
              return true;
            }
            return false;
          });

          if loved {
            square.blood += 1;
          } else if square.blood > 2 {
            let mut love = false;
            self.check_nearby(x, y, 2, 2, |cell| {
              if cell.concept == Concept::Dogwood {
                love = true;
                return true;
              }
              return false;
            });
            if love {
              square.concept = Concept::Rose;
            } else {
              to_dust(square);
            }
          } else {
            to_dust(square);
          }
        } else if square.concept == Concept::Soil {
          if count == 3 {
            // Life
            square.concept = Concept::Sunflower;
          } else if count == 8 {
            // Compassion
            square.concept = Concept::Rose;
          }
        }
      } else if square.concept == Concept::Rose {
        let mut suffering = 0;
        self.check_nearby(x, y, 2, 1, |cell| {
          suffering += cell.blood;
          return false;
        });

        if suffering >= 12 {
          // Sacrifice
          square.concept = Concept::Dogwood;
          square.blood = -50;
        }
      } else if square.concept == Concept::Dogwood {
        square.blood += 1;
        if square.blood >= 0 {
          // Forgotten
          to_dust(square);
        }
      }
    }
    self.playground = new_playground;
  }
}

impl<'a> IntoIterator for &'a GameOfLife {
  type Item = &'a CellState;
  type IntoIter = ::std::slice::Iter<'a, CellState>;
  fn into_iter(self) -> ::std::slice::Iter<'a, CellState> {
    self.playground.iter()
  }
}
