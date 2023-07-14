use std::fmt;

#[derive(Debug)]
pub enum Rule {
  RawPointerUsage(String),
  MemoryFree(String),
  UnknownRule(String),
}

impl fmt::Display for Rule {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::RawPointerUsage(s) => write!(
        f,
        "[Raw Pointer Usage Error][Invalid usage of raw pointer]\n>>>>>\n{}\n<<<<<",
        s
      ),
      Self::MemoryFree(s) => write!(
        f,
        "[Memory Free Error][Error with memory deallocation]\n>>>>>\n{}\n<<<<<",
        s
      ),
      Self::UnknownRule(s) => write!(f, "[Unknown Rule Error]\n>>>>>\n{}\n<<<<<", s),
    }
  }
}
