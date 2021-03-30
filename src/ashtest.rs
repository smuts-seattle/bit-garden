use crate::game::CellState;
use crate::game::Concept;
use std::io::{stdin, stdout, Read, Write};
use std::path::PathBuf;
use std::rc::Rc;
use wyzoid::high;
use wyzoid::low::vkstate::VulkanState;

fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}

pub struct Runner {
    shader: PathBuf,
    vulkan: Rc<VulkanState>,
}

impl Runner {
    pub fn new() -> Runner {
        Runner {
            shader: PathBuf::from("target/game.spv"),
            vulkan: Rc::new(wyzoid::low::vkstate::init_vulkan()),
        }
    }

    pub fn run(&self, input: &Vec<CellState>, world_width: u32) -> Vec<CellState> {
        /*for i in input {
            print!("{};", i.blood as i32);
        }
        pause();*/
        let mut inp: Option<Vec<CellState>> = None;

        for i in 0..10 {
            let inp_ref = match &inp {
                Some(x) => x,
                None => &input,
            };
            let world_width_buffer = vec![CellState {
                concept: if i % 2 == 0 {
                    Concept::Soil
                } else {
                    Concept::Sunflower
                },
                blood: world_width as i32,
            }];

            // We create the compute job.
            // Since our shader has a local work size of 64, we divide the number of data by 64 for the dispatch.
            let mut job = high::job::JobBuilder::new()
                .add_buffer(inp_ref, 0, 0)
                .add_buffer(inp_ref, 0, 1)
                .add_buffer(&world_width_buffer, 0, 2)
                .add_shader(&self.shader)
                .add_dispatch(((input.len() / 32) as u32, 1, 1))
                .build(self.vulkan.clone());

            job.execute();

            while job.status() == wyzoid::high::job::JobStatus::EXECUTING {
                job.wait_until_idle(1 * 1000 * 1000 * 1000);
            }

            //let timings = job.get_timing();
            //println!("=======\ntimings:\n{}\n=======", timings);

            inp = Some(job.get_output().unwrap().remove(1));
        }

        return inp.unwrap();
        /*for i in &shader_output[1] {
            print!("{}, {};", i.concept as i32, i.blood as i32);
        }

        println!("");
        pause();*/
    }
}
