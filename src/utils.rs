use crate::MgError;
use regex::Regex;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

pub fn new_writer(output: &Option<PathBuf>) -> Result<Box<dyn Write>, MgError> {
  match output {
    None => Ok(Box::new(io::stderr())),
    Some(path) => Ok(Box::new(
      File::create(path).map_err(|e| MgError::PathError(e.to_string()))?,
    )),
  }
}

pub fn check_cargo() -> Result<(), MgError> {
  match Command::new("cargo").args(["--version"]).output() {
    Err(e) => Err(MgError::CargoError(format!("{:?}: {}", e.kind(), e))),
    Ok(out) => {
      if !out.status.success() {
        Err(MgError::CargoError(
          String::from_utf8_lossy(&out.stderr).to_string(),
        ))
      } else {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if !stdout.contains("nightly") {
          Err(MgError::CargoError(
            [
              "Nightly toolchain is needed.",
              "Note: You can install the nightly toolchain with the command:",
              "  `rustup toolchain install nightly`",
              "Note: If the nightly toolchain is installed, you can override it for current project:",
              "  `rustup override set nightly`"
            ]
              .join("\n"),
          ))
        } else {
          Ok(())
        }
      }
    }
  }
}

pub fn check_miri() -> Result<(), MgError> {
  match Command::new("cargo").args(["miri", "--version"]).output() {
    Err(e) => Err(MgError::MiriError(format!("{:?}: {}", e.kind(), e))),
    Ok(out) => {
      if !out.status.success() {
        Err(MgError::MiriError(
          String::from_utf8_lossy(&out.stderr).to_string(),
        ))
      } else {
        Ok(())
      }
    }
  }
}

pub fn extract_errors(output: &str) -> Vec<String> {
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
