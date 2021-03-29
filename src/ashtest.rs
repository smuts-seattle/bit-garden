use crate::game::CellState;
use std::io::{stdin, stdout, Read, Write};
use std::path::PathBuf;
use std::rc::Rc;
use wyzoid::high;

fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}

pub fn run(input: &Vec<CellState>) {
    for i in input {
        print!("{};", i.blood as i32);
    }
    pause();
    let output = input.to_vec();
    // We use a shader that compute sinus and cosinus using taylor series.
    // Buffer one will be sinus, buffer two will be cosinus.
    let shader = PathBuf::from("target/game.spv");

    let vulkan = Rc::new(wyzoid::low::vkstate::init_vulkan());

    // We create the compute job.
    // Since our shader has a local work size of 64, we divide the number of data by 64 for the dispatch.
    let mut job = high::job::JobBuilder::new()
        .add_buffer(input, 0, 0)
        .add_buffer(&output, 0, 1)
        .add_shader(&shader)
        .add_dispatch((input.len() as u32, 1, 1))
        .build(vulkan);

    job.execute();
    while job.status() == wyzoid::high::job::JobStatus::EXECUTING {
        job.wait_until_idle(1 * 1000 * 1000 * 1000);
    }
    let shader_output = job.get_output().unwrap();
    let timings = job.get_timing();
    println!("=======\ntimings:\n{}\n=======", timings);

    for i in &shader_output[1] {
        print!("{}, {};", i.concept as i32, i.blood as i32);
    }

    println!("");
    pause();
}
