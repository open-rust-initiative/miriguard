use assert_cmd::Command;
use predicates::prelude::predicate;
use std::error::Error;

type TestResult<T> = Result<T, Box<dyn Error>>;

const PRG: &'static str = "miriguard";

#[test]
fn miriguard_without_subcmd() {
  let assert = Command::cargo_bin(PRG).unwrap().assert();

  assert.failure().stderr(predicate::str::contains("Usage"));
}

#[test]
fn miriguard_with_unrecognized_subcmd() {
  let subcmd = "foo";
  let assert = Command::cargo_bin(PRG).unwrap().arg(subcmd).assert();

  assert.failure().stderr(predicate::str::contains("Usage"));
}

#[test]
fn unrecognized_run_arg() -> TestResult<()> {
  let assert = Command::cargo_bin(PRG)
    .unwrap()
    .current_dir("tests/hello/")
    .args(["run", "main"])
    .assert();

  assert.failure().stderr(predicate::str::is_match(
    "error: unexpected argument .+ found",
  )?);
  Ok(())
}

#[test]
fn run_hello() {
  let assert = Command::cargo_bin(PRG)
    .unwrap()
    .current_dir("tests/hello/")
    .arg("run")
    .assert();

  assert
    .failure()
    .stderr(predicate::str::starts_with("[Miri Error]: "));
}

#[test]
fn run_bin_hello() {
  let assert = Command::cargo_bin(PRG)
    .unwrap()
    .current_dir("tests/hello/")
    .args(["run", "--bin", "uninitial"])
    .assert();

  assert.success().stderr(predicate::str::starts_with(
    "ERROR: [Raw Pointer Usage Error][Invalid usage of raw pointer]\n>>>>>\n",
  ));
}

#[test]
fn run_example_hello() {
  let assert = Command::cargo_bin(PRG)
    .unwrap()
    .current_dir("tests/hello/")
    .args(["run", "--example", "uninitial"])
    .assert();

  assert.success().stderr(predicate::str::starts_with(
    "ERROR: [Raw Pointer Usage Error][Invalid usage of raw pointer]\n>>>>>\n",
  ));
}

#[test]
fn test_memory_leaking() {
  let assert = Command::cargo_bin(PRG)
    .unwrap()
    .current_dir("tests/memory_deallocation")
    .args(["test", "memory_leaking"])
    .assert();

  assert.success().stderr(predicate::str::starts_with(
    "ERROR: [Memory Free Error][Error with memory deallocation]\n>>>>>\n",
  ));
}

#[test]
fn test_double_free() {
  let assert = Command::cargo_bin(PRG)
    .unwrap()
    .current_dir("tests/memory_deallocation")
    .args(["test", "double_free"])
    .assert();

  assert.success().stderr(predicate::str::starts_with(
    "ERROR: [Memory Free Error][Error with memory deallocation]\n>>>>>\n",
  ));
}

#[test]
fn test_uninitialized_pointer() {
  let assert = Command::cargo_bin(PRG)
    .unwrap()
    .current_dir("tests/raw_point_validity")
    .args(["test", "uninitialized_pointer"])
    .assert();

  assert.success().stderr(predicate::str::starts_with(
    "ERROR: [Raw Pointer Usage Error][Invalid usage of raw pointer]\n>>>>>\n",
  ));
}

#[test]
fn test_return_stack_address() {
  let assert = Command::cargo_bin(PRG)
    .unwrap()
    .current_dir("tests/raw_point_validity")
    .args(["test", "stack_address"])
    .assert();

  assert.success().stderr(predicate::str::contains(
    "ERROR: [Raw Pointer Usage Error][Invalid usage of raw pointer]\n>>>>>\n",
  ));
}

#[test]
fn run_multi_miri_tests_without_testname() {
  let assert = Command::cargo_bin(PRG)
    .unwrap()
    .current_dir("tests/raw_point_validity")
    .arg("test")
    .assert();

  assert.success().stderr(
    predicate::str::contains("ERROR: [Raw Pointer Usage Error][Invalid usage of raw pointer]")
      .count(2),
  );
}

#[test]
fn run_multi_miri_test_with_names() {
  let assert = Command::cargo_bin(PRG)
    .unwrap()
    .current_dir("tests/raw_point_validity")
    .args(["test", "stack", "uninitialized"])
    .assert();

  assert.success().stderr(
    predicate::str::contains("ERROR: [Raw Pointer Usage Error][Invalid usage of raw pointer]")
      .count(2),
  );
}

#[test]
fn unsupported_operation() {
  let assert = Command::cargo_bin(PRG)
    .unwrap()
    .current_dir("tests/unsupported_operation")
    .arg("run")
    .assert();

  assert.success().stderr("");
}
