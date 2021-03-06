extern crate wyzoid;
use std::path::PathBuf;
use std::rc::Rc;
use wyzoid::{high, utils};

const DATA_LEN: usize = 64;

fn main() {
    // We generate 64 random float between 0.0 and 1.0.
    let input1 = utils::rand_vec::<f32>(256, 0.0, 1.0);
    let input2 = utils::rand_vec::<f32>(64, 0.0, 1.0);

    // We use a shader that compute sinus and cosinus using taylor series.
    // Buffer one will be sinus, buffer two will be cosinus.
    let shader = PathBuf::from("examples/shaders/bin/examples/reduce.cs.spirv");

    let vulkan = Rc::new(wyzoid::low::vkstate::init_vulkan());

    // We create the compute job.
    // Since our shader has a local work size of 64, we divide the number of data by 64 for the dispatch.
    let mut job = high::job::JobBuilder::new()
        .add_buffer(&input1, 0, 0)
        .add_buffer(&input2, 0, 1)
        .add_shader(&shader)
        .add_dispatch((4, 1, 1))
        .build(vulkan);

    job.execute();
    while job.status() == wyzoid::high::job::JobStatus::EXECUTING {
        job.wait_until_idle(1 * 1000 * 1000 * 1000);
    }
    let shader_output = job.get_output().unwrap();
    let timings = job.get_timing();
    println!("=======\ntimings:\n{}\n=======", timings);

    for i in input1 {
        print!("{};", i);
    }

    println!("");

    for i in input2 {
        print!("{};", i);
    }

    println!("");

    for i in &shader_output[1] {
        print!("{};", i);
    }

    println!("");
}
