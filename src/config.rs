use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(propagate_version = true)]
pub struct Config {
  #[command(subcommand)]
  pub command: Commands,
  #[arg(short, long)]
  pub output: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
  Run(RunArgs),
  Test(TestArgs),
}

#[derive(Args, Debug)]
pub struct RunArgs {
  #[arg(group = "run-target", long)]
  pub bin: Option<String>,
  #[arg(group = "run-target", long)]
  pub example: Option<String>,
}

#[derive(Args, Debug)]
pub struct TestArgs {
  pub testname: Option<String>,
}
