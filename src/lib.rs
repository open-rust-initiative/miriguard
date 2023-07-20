mod config;
mod error;
mod rule;
mod utils;

pub use config::Config;
pub use error::MgError;
pub use rule::Rule;
use std::io::{BufRead, Write};

use config::{Commands, RunArgs, TestArgs};
use std::process::Command;

pub fn run(config: &Config) -> Result<(), MgError> {
  let mut writer = utils::new_writer(&config.output)?;
  let mut write_rule = |res: Vec<Rule>| -> Result<(), MgError> {
    if res.len() == 1 && matches!(res[0], Rule::UnknownRule(_)) {
      match &res[0] {
        Rule::UnknownRule(s) if s.contains("Undefined Behavior: ") => {
          writeln!(writer, "ERROR: {}\n", res[0]).unwrap();
          Ok(())
        }
        Rule::UnknownRule(s) => Err(MgError::MiriError(s.to_string())),
        _ => unreachable!(),
      }
    } else {
      res.iter().for_each(|r| match r {
        Rule::UnknownRule(s) if s.contains("previous error") => {}
        Rule::UnknownRule(s) if s.contains("test failed, to rerun pass") => {}
        Rule::UnknownRule(s) if s.contains("unsupported operation: ") => {}
        rule => writeln!(writer, "ERROR: {rule}\n").unwrap(),
      });
      Ok(())
    }
  };

  utils::check_cargo()?;
  utils::check_miri()?;
  match &config.command {
    Commands::Run(args) => miri_run(args, &mut write_rule),
    Commands::Test(args) => miri_test(args, &mut write_rule),
  }
}

fn miri_run<F>(args: &RunArgs, f: &mut F) -> Result<(), MgError>
where
  F: FnMut(Vec<Rule>) -> Result<(), MgError>,
{
  if let Some(bin) = &args.bin {
    let args = ["miri", "run", "--bin", bin];
    exec_miri_command(&args, f)
  } else if let Some(example) = &args.example {
    let args = ["miri", "run", "--example", example];
    exec_miri_command(&args, f)
  } else {
    let args = ["miri", "run"];
    exec_miri_command(&args, f)
  }
}

fn miri_test<F>(args: &TestArgs, f: &mut F) -> Result<(), MgError>
where
  F: FnMut(Vec<Rule>) -> Result<(), MgError>,
{
  let test_names = match &args.testname {
    None => get_test_list()?,
    Some(names) => names.clone(),
  };
  for test in &test_names {
    exec_miri_command(&["miri", "test", test], f)?;
  }
  Ok(())
}

fn get_test_list() -> Result<Vec<String>, MgError> {
  match Command::new("cargo")
    .args(["test", "--", "--list"])
    .output()
  {
    Err(e) => Err(MgError::CargoError(format!("{:?}: {}", e.kind(), e))),
    Ok(out) => {
      if !out.status.success() {
        Err(MgError::CargoError(
          String::from_utf8_lossy(&out.stderr).to_string(),
        ))
      } else {
        let tests = out
          .stdout
          .lines()
          .filter_map(|line| match line {
            Err(_) => None,
            Ok(ln) => {
              if ln.ends_with(": test") {
                Some(ln.strip_suffix(": test").unwrap().to_string())
              } else {
                None
              }
            }
          })
          .collect::<Vec<_>>();
        Ok(tests)
      }
    }
  }
}

fn exec_miri_command<F>(args: &[&str], write: &mut F) -> Result<(), MgError>
where
  F: FnMut(Vec<Rule>) -> Result<(), MgError>,
{
  match Command::new("cargo").args(args).output() {
    Err(e) => Err(MgError::MiriError(format!("{:?}: {}", e.kind(), e))),
    Ok(out) => {
      if !out.status.success() {
        check_miri_error_output(&String::from_utf8_lossy(&out.stderr), write)
      } else {
        print!("{}", String::from_utf8_lossy(&out.stderr));
        Ok(())
      }
    }
  }
}

fn check_miri_error_output<F>(miri_output: &str, write: &mut F) -> Result<(), MgError>
where
  F: FnMut(Vec<Rule>) -> Result<(), MgError>,
{
  if miri_output.starts_with("Preparing a sysroot") {
    let first_line = miri_output.lines().next().unwrap();
    if first_line.contains("error") {
      return Err(MgError::MiriError(first_line.to_string()));
    } else {
      println!("{first_line}");
    }
  }

  let err_msgs = utils::extract_errors(miri_output);
  let res = err_msgs
    .into_iter()
    .map(|e| Rule::match_error(&e))
    .collect::<Vec<_>>();
  write(res)
}
