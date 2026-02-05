use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignalError {
    #[error("Invalid instruction: {0}")]
    InvalidInstructionError(String),
    #[error("Fail to send signal: {0}")]
    SendSignalError(String),
}
