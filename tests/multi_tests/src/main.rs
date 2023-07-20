#[cfg(test)]
mod tests {

  #[test]
  fn access_returned_stack_address() {
    fn return_stack_address() -> *const i32 {
      let value = 123_i32;
      &value as *const i32
    }

    let p = return_stack_address();
    println!("{}", unsafe { *p });
  }

  #[test]
  fn double_free() {
    let buf = unsafe { libc::malloc(1) };
    unsafe { libc::free(buf) };
    unsafe { libc::free(buf) };
  }
}
