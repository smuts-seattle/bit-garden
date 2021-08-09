use regex::Regex;

#[derive(Clone, Copy)]
pub struct Args {
  pub size: u32,
  pub pixel_size: u32,
  pub update_rate: u32,
  pub show_graphics: bool,
  pub draw_rate: u32,
  pub show_fps: bool,
}

pub fn parse_args(args: Vec<String>) -> Args {
  let mut result = Args {
    size: 100,
    pixel_size: 4,
    update_rate: 2,
    show_graphics: false,
    draw_rate: 2,
    show_fps: true,
  };

  let set = Regex::new(
    r"(size)=(.*)|(pixel_size)=(.*)|(update_rate)=(.*)|(show_graphics)=(.*)|(draw_rate)=(.*)|(show_fps)=(.*)",
  )
  .unwrap();

  for arg in args {
    println!("arg: {}", arg);
    let captures: Vec<&str> = set
      .captures(&arg)
      .expect(&format!("Unrecognized argument: {}", arg))
      .iter()
      .filter(|&s| s.is_some())
      .map(|s| s.unwrap().as_str())
      .collect();

    let arg_name = captures[1];
    println!("cap: {}", arg_name);
    let arg_value = captures[2];

    match arg_name {
      "size" => {
        result.size = arg_value
          .parse::<u32>()
          .expect(&format!("Could not parse size: {}", arg_value))
      }
      "pixel_size" => {
        result.pixel_size = arg_value
          .parse::<u32>()
          .expect(&format!("Could not parse pixel_size: {}", arg_value))
      }
      "update_rate" => {
        result.update_rate = arg_value
          .parse::<u32>()
          .expect(&format!("Could not parse update_rate: {}", arg_value))
      }
      "show_graphics" => {
        result.show_graphics = arg_value
          .parse::<bool>()
          .expect(&format!("Could not parse show_graphics: {}", arg_value))
      }
      "draw_rate" => {
        result.draw_rate = arg_value
          .parse::<u32>()
          .expect(&format!("Could not parse draw_rate: {}", arg_value))
      }
      "show_fps" => {
        result.show_fps = arg_value
          .parse::<bool>()
          .expect(&format!("Could not parse show_fps: {}", arg_value))
      }

      _ => {}
    }
  }

  return result;
}
