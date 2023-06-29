use regex::Regex;
use std::process::Command;
use std::sync::OnceLock;
use std::{env, process, str};
use thiserror::Error;

#[derive(Error, Debug)]
enum MiriGuardError {
  #[error("{0}")]
  Cargo(String),
  #[error("{0}")]
  Miri(String),
  #[error("`miriguard` needs to be called with a subcommand (`run`, `test`)")]
  MissSubCmd,
  #[error("unrecognized subcommand `{0}`")]
  WrongSubCmd(String),
  #[error("error with using invalid raw pointer >>>>>\n{0}\n<<<<<")]
  RawPointerUsage(String),
  #[error("error with memory deallocation >>>>>\n{0}\n<<<<<")]
  MemoryFree(String),
}

fn main() {
  check_cargo().unwrap_or_else(|e| {
    eprintln!("Error: {e}");
    process::exit(1);
  });
  check_and_exec_miri().unwrap_or_else(|e| {
    eprintln!("Error: {e}");
    process::exit(1);
  });
}

fn check_cargo() -> Result<(), MiriGuardError> {
  match Command::new("cargo").args(["+nightly", "-vV"]).output() {
    Err(e) => Err(MiriGuardError::Cargo(format!("{:?}: {}", e.kind(), e))),
    Ok(out) => {
      if !out.status.success() {
        Err(MiriGuardError::Cargo(
          str::from_utf8(&out.stderr).unwrap().to_string(),
        ))
      } else {
        Ok(())
      }
    }
  }
}

fn check_and_exec_miri() -> Result<(), MiriGuardError> {
  match Command::new("cargo")
    .args(["+nightly", "miri", "--version"])
    .output()
  {
    Err(e) => Err(MiriGuardError::Miri(format!("{:?}: {}", e.kind(), e))),
    Ok(out) => {
      if !out.status.success() {
        Err(MiriGuardError::Miri(
          str::from_utf8(&out.stderr).unwrap().to_string(),
        ))
      } else {
        let args: Vec<_> = env::args().skip(1).collect();
        match args.get(0).map(|s| s.as_str()) {
          Some("run") | Some("test") => exec_miri(args),
          Some(s) => Err(MiriGuardError::WrongSubCmd(s.to_string())),
          None => Err(MiriGuardError::MissSubCmd),
        }
      }
    }
  }
}

fn exec_miri(args: Vec<String>) -> Result<(), MiriGuardError> {
  let args: Vec<_> = ["+nightly", "miri"]
    .into_iter()
    .map(String::from)
    .chain(args.into_iter())
    .collect();

  match Command::new("cargo").args(args).output() {
    Err(e) => Err(MiriGuardError::Miri(format!("{:?}: {}", e.kind(), e))),
    Ok(out) => {
      if !out.status.success() {
        check_miri_error_output(str::from_utf8(&out.stderr).unwrap())
      } else {
        print!("{}", str::from_utf8(&out.stderr).unwrap());
        Ok(())
      }
    }
  }
}

fn check_miri_error_output(miri_output: &str) -> Result<(), MiriGuardError> {
  let err_msgs = extract_errors(miri_output);
  for msg in err_msgs {
    match_error_with_guidelines(&msg)?;
  }
  Ok(())
}

fn match_error_with_guidelines(error: &str) -> Result<(), MiriGuardError> {
  static RE_MEM_FREE: OnceLock<Regex> = OnceLock::new();
  static RE_PTR_USAGE: OnceLock<Regex> = OnceLock::new();
  let re_mem_free = RE_MEM_FREE.get_or_init(|| {
    Regex::new(r"error: Undefined Behavior: pointer to alloc\d+ was dereferenced after this allocation got free").unwrap()
  });
  let re_ptr_usage =
    RE_PTR_USAGE.get_or_init(|| {
      Regex::new(r"error: Undefined Behavior: dereferencing pointer failed: null pointer is a dangling pointer").unwrap()
    });

  if re_mem_free.is_match(error) {
    Err(MiriGuardError::MemoryFree(error.to_string()))
  } else if re_ptr_usage.is_match(error) {
    Err(MiriGuardError::RawPointerUsage(error.to_string()))
  } else {
    Ok(())
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
  use crate::extract_errors;

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
