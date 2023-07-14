use thiserror::Error;

#[derive(Error, Debug)]
pub enum MgError {
  #[error("[Cargo Error]: {0}")]
  CargoError(String),
  #[error("[Miri Error]: {0}")]
  MiriError(String),
  #[error("[Path Error]: {0}")]
  PathError(String),
}
