#[cfg(test)]
mod tests {
  #[derive(Debug)]
  struct Foo<'a> {
    ptr: &'a i32,
  }

  #[test]
  fn uninitialized_pointer() {
    let foo_ptr: *mut Foo = std::ptr::null_mut();
    println!("{:?}", unsafe { (*foo_ptr).ptr });
    assert!(true);
  }

  #[test]
  fn access_returned_stack_address() {
    fn return_stack_address() -> *const i32 {
      let value = 123_i32;
      &value as *const i32
    }

    let p = return_stack_address();
    println!("{}", unsafe { *p });
  }
}
