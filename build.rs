use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
  Command::new("bash")
    .args(&["make_resources.sh"])
    .status()
    .unwrap();
}
