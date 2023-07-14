mod config;
mod error;
mod rule;

pub use config::Config;
pub use error::MgError;
pub use rule::Rule;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;

use config::{Commands, RunArgs, TestArgs};
use regex::Regex;
use std::process::Command;
use std::sync::OnceLock;

pub fn run(config: &Config) -> Result<(), MgError> {
  let mut writer = new_writer(&config.output)?;

  check_cargo()?;
  let res = check_and_exec_miri(config)?;
  if res.len() == 1 && matches!(res[0], Rule::UnknownRule(_)) {
    post_process_unknown_rule(&res[0], writer)
  } else {
    res.iter().for_each(|r| match r {
      Rule::UnknownRule(s) if s.contains("previous error") => {}
      Rule::UnknownRule(s) if s.contains("test failed, to rerun pass") => {}
      rule => writeln!(writer, "ERROR: {rule}").unwrap(),
    });
    Ok(())
  }
}

fn new_writer(output: &Option<PathBuf>) -> Result<Box<dyn Write>, MgError> {
  match output {
    None => Ok(Box::new(io::stderr())),
    Some(path) => Ok(Box::new(
      File::create(path).map_err(|e| MgError::PathError(e.to_string()))?,
    )),
  }
}

fn post_process_unknown_rule(rule: &Rule, mut f: impl Write) -> Result<(), MgError> {
  match rule {
    Rule::UnknownRule(s) if s.contains("Undefined Behavior: ") => {
      writeln!(f, "ERROR: {rule}").unwrap()
    }
    Rule::UnknownRule(s) => return Err(MgError::MiriError(s.to_string())),
    _ => unreachable!(),
  }
  Ok(())
}

fn check_cargo() -> Result<(), MgError> {
  match Command::new("cargo").args(["+nightly", "-vV"]).output() {
    Err(e) => Err(MgError::CargoError(format!("{:?}: {}", e.kind(), e))),
    Ok(out) => {
      if !out.status.success() {
        Err(MgError::CargoError(
          String::from_utf8_lossy(&out.stderr).to_string(),
        ))
      } else {
        Ok(())
      }
    }
  }
}

fn check_and_exec_miri(config: &Config) -> Result<Vec<Rule>, MgError> {
  match Command::new("cargo")
    .args(["+nightly", "miri", "--version"])
    .output()
  {
    Err(e) => Err(MgError::MiriError(format!("{:?}: {}", e.kind(), e))),
    Ok(out) => {
      if !out.status.success() {
        Err(MgError::MiriError(
          String::from_utf8_lossy(&out.stderr).to_string(),
        ))
      } else {
        match &config.command {
          Commands::Run(args) => miri_run(args),
          Commands::Test(args) => miri_test(args),
        }
      }
    }
  }
}

fn miri_run(args: &RunArgs) -> Result<Vec<Rule>, MgError> {
  if let Some(bin) = &args.bin {
    let args = ["+nightly", "miri", "run", "--bin", bin];
    exec_miri_command(&args)
  } else if let Some(example) = &args.example {
    let args = ["+nightly", "miri", "run", "--example", example];
    exec_miri_command(&args)
  } else {
    let args = ["+nightly", "miri", "run"];
    exec_miri_command(&args)
  }
}

fn miri_test(args: &TestArgs) -> Result<Vec<Rule>, MgError> {
  match &args.testname {
    None => exec_miri_command(&["+nightly", "miri", "test"]),
    Some(name) => exec_miri_command(&["+nightly", "miri", "test", name]),
  }
}

fn exec_miri_command(args: &[&str]) -> Result<Vec<Rule>, MgError> {
  match Command::new("cargo").args(args).output() {
    Err(e) => Err(MgError::MiriError(format!("{:?}: {}", e.kind(), e))),
    Ok(out) => {
      if !out.status.success() {
        check_miri_error_output(&String::from_utf8_lossy(&out.stderr))
      } else {
        print!("{}", String::from_utf8_lossy(&out.stderr));
        Ok(vec![])
      }
    }
  }
}

fn check_miri_error_output(miri_output: &str) -> Result<Vec<Rule>, MgError> {
  if miri_output.starts_with("Preparing a sysroot") {
    let first_line = miri_output.lines().next().unwrap();
    if first_line.contains("error") {
      return Err(MgError::MiriError(first_line.to_string()));
    } else {
      println!("{first_line}");
    }
  }

  let err_msgs = extract_errors(miri_output);
  Ok(
    err_msgs
      .into_iter()
      .map(|e| match_error_with_guidelines(&e))
      .collect::<Vec<_>>(),
  )
}

fn match_error_with_guidelines(error: &str) -> Rule {
  static MEM_LEAK: OnceLock<Regex> = OnceLock::new();
  static DEREF_NULL_PTR: OnceLock<Regex> = OnceLock::new();
  static DEREF_AFTER_FREE: OnceLock<Regex> = OnceLock::new();
  let mem_leak = MEM_LEAK.get_or_init(|| Regex::new(r"error: memory leaked: alloc\d+").unwrap());
  let deref_null_ptr =
    DEREF_NULL_PTR.get_or_init(|| {
      Regex::new(r"error: Undefined Behavior: dereferencing pointer failed: null pointer is a dangling pointer").unwrap()
    });
  let deref_after_free =
    DEREF_AFTER_FREE.get_or_init(|| {
      Regex::new(r"error: Undefined Behavior: pointer to alloc\d+ was dereferenced after this allocation got free").unwrap()
    });

  if mem_leak.is_match(error) {
    Rule::MemoryFree(error.to_string())
  } else if deref_null_ptr.is_match(error) {
    Rule::RawPointerUsage(error.to_string())
  } else if deref_after_free.is_match(error) {
    if error.contains("libc::free(") {
      Rule::MemoryFree(error.to_string())
    } else {
      Rule::RawPointerUsage(error.to_string())
    }
  } else {
    Rule::UnknownRule(error.to_string())
  }
}

fn extract_errors(output: &str) -> Vec<String> {
  static RE_ERROR: OnceLock<Regex> = OnceLock::new();
  let regex_err = RE_ERROR.get_or_init(|| Regex::new(r"(?m)^(error: (?s:.)*)").unwrap());
  output
    .split("\n\n")
    .filter_map(|s| regex_err.find(s))
    .map(|cap| cap.as_str().to_string())
    .collect()
}

#[cfg(test)]
mod tests {
  use super::extract_errors;

  #[test]
  fn extract_errors_from_miri_stderr() {
    let error_output = "   Compiling playground v0.0.1 (/playground)
    Finished dev [unoptimized + debuginfo] target(s) in 0.29s
     Running `/playground/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo-miri runner target/miri/x86_64-unknown-linux-gnu/debug/playground`
error: Undefined Behavior: trying to join an already joined thread
  --> src/main.rs:14:20
   |
14 |         assert_eq!(libc::pthread_join(native, ptr::null_mut()), 0); //~ ERROR: Undefined Behavior: trying to join an already joined thread
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ trying to join an already joined thread
   |
   = help: this indicates a bug in the program: it performed an invalid operation, and caused Undefined Behavior
   = help: see https://doc.rust-lang.org/nightly/reference/behavior-considered-undefined.html for further information
   = note: BACKTRACE:
   = note: inside `main` at src/main.rs:14:20: 14:63

note: some details are omitted, run with `MIRIFLAGS=-Zmiri-backtrace=full` for a verbose backtrace

error: aborting due to previous error
";
    let errors = extract_errors(error_output);

    assert_eq!(errors.len(), 2);
    assert_eq!(
      errors[0].as_str(),
      "error: Undefined Behavior: trying to join an already joined thread
  --> src/main.rs:14:20
   |
14 |         assert_eq!(libc::pthread_join(native, ptr::null_mut()), 0); //~ ERROR: Undefined Behavior: trying to join an already joined thread
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ trying to join an already joined thread
   |
   = help: this indicates a bug in the program: it performed an invalid operation, and caused Undefined Behavior
   = help: see https://doc.rust-lang.org/nightly/reference/behavior-considered-undefined.html for further information
   = note: BACKTRACE:
   = note: inside `main` at src/main.rs:14:20: 14:63"
    );
    assert_eq!(
      errors[1].as_str(),
      "error: aborting due to previous error\n"
    );
  }
}
