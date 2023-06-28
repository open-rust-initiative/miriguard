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
}
