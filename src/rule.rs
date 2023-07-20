use regex::Regex;
use std::fmt;
use std::sync::OnceLock;

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

impl Rule {
  pub fn match_error(error: &str) -> Rule {
    static MEM_LEAK: OnceLock<Regex> = OnceLock::new();
    static DEREF_NULL_PTR: OnceLock<Regex> = OnceLock::new();
    static DEREF_AFTER_FREE: OnceLock<Regex> = OnceLock::new();
    let mem_leak =
      MEM_LEAK.get_or_init(|| Regex::new(r"error: memory leaked|leaked memory").unwrap());
    let deref_null_ptr =
      DEREF_NULL_PTR.get_or_init(|| {
        Regex::new(r"error: Undefined Behavior: dereferencing pointer failed: null pointer is a dangling pointer").unwrap()
      });
    let deref_after_free =
      DEREF_AFTER_FREE.get_or_init(|| {
        Regex::new(r"error: Undefined Behavior: pointer to alloc\d+ was dereferenced after this allocation got free").unwrap()
      });

    if mem_leak.is_match(error) {
      Rule::MemoryFree(error.to_string())
    } else if deref_null_ptr.is_match(error) {
      Rule::RawPointerUsage(error.to_string())
    } else if deref_after_free.is_match(error) {
      if error.contains("libc::free(") {
        Rule::MemoryFree(error.to_string())
      } else {
        Rule::RawPointerUsage(error.to_string())
      }
    } else {
      Rule::UnknownRule(error.to_string())
    }
  }
}
