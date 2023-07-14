fn main() {
  let ptr: *mut u8 = std::ptr::null_mut();
  println!("{:?}", unsafe { *ptr });
}