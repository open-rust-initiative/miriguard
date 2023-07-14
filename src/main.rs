use clap::Parser;
use miriguard::Config;
use std::process;

fn main() {
  let config = Config::parse();
  if let Err(e) = miriguard::run(&config) {
    eprintln!("{e}");
    process::exit(1);
  }
}
