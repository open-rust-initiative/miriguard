fn main() {
  let t = 1689853489_i64;
  unsafe {
    let tm = libc::localtime(&t as *const libc::time_t);
    let year = (*tm).tm_year;
    let month = (*tm).tm_mon;
    let day = (*tm).tm_mday;
    println!("{year}-{month}-{day}");
  }
}
