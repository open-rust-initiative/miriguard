use assert_cmd::Command;
use predicates::prelude::predicate;

#[test]
fn miriguard_without_subcmd() {
  let assert = Command::cargo_bin("miriguard").unwrap().assert();

  assert.failure().stderr(
    "Error: `miriguard` needs to be called with a subcommand (`run`, `test`)\n".to_string(),
  );
}

#[test]
fn miriguard_with_unrecognized_subcmd() {
  let subcmd = "foo";
  let assert = Command::cargo_bin("miriguard")
    .unwrap()
    .arg(subcmd)
    .assert();

  assert
    .failure()
    .stderr(format!("Error: unrecognized subcommand `{subcmd}`\n"));
}

#[test]
fn crate_hello() {
  let assert = Command::cargo_bin("miriguard")
    .unwrap()
    .current_dir("tests/hello/")
    .arg("run")
    .assert();
  assert.success();
}

#[test]
fn crate_memory_deallocation() {
  let assert = Command::cargo_bin("miriguard")
    .unwrap()
    .current_dir("tests/memory_deallocation")
    .args(["test", "memory_leaking"])
    .assert();

  assert.failure().stderr(predicate::str::starts_with(
    "Error: error with memory deallocation >>>",
  ));

  let assert = Command::cargo_bin("miriguard")
    .unwrap()
    .current_dir("tests/memory_deallocation")
    .args(["test", "double_free"])
    .assert();

  assert.failure().stderr(predicate::str::starts_with(
    "Error: error with memory deallocation >>>",
  ));
}

#[test]
fn crate_raw_point_validity() {
  let assert = Command::cargo_bin("miriguard")
    .unwrap()
    .current_dir("tests/raw_point_validity")
    .args(["test", "initial"])
    .assert();

  assert.failure().stderr(predicate::str::starts_with(
    "Error: error with using invalid raw pointer >>>",
  ));

  let assert = Command::cargo_bin("miriguard")
    .unwrap()
    .current_dir("tests/raw_point_validity")
    .args(["test", "stack"])
    .assert();

  assert.failure().stderr(predicate::str::starts_with(
    "Error: error with using invalid raw pointer >>>",
  ));
}
