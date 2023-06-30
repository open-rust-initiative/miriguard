#![allow(dead_code)]

#[cfg(test)]
mod tests {
  const BUF_SIZE: usize = 100;

  #[test]
  fn memory_leaking() {
    let _buf = unsafe { libc::malloc(BUF_SIZE) };
  }

  #[test]
  fn double_free() {
    let buf = unsafe { libc::malloc(BUF_SIZE) };
    unsafe { libc::free(buf) };
    unsafe { libc::free(buf) };
  }
}
