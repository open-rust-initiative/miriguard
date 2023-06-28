#[cfg(test)]
mod tests {
  #[test]
  fn free_memory() {
    const BUF_SIZE: usize = 100;
    unsafe {
      let buf = libc::malloc(BUF_SIZE) as *mut libc::c_char;

      libc::free(buf as *mut libc::c_void);

      if !buf.is_null() {
        println!("{:?}", *buf);
      }
    }
    assert!(true);
  }
}
