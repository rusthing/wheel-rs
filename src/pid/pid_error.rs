use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PidError {
    #[error("invalid PID file path: {0}")]
    InvalidPidFilePath(PathBuf),
    #[error("Fail to open PID file: {0}")]
    OpenPidFileError(String),
    #[error("Fail to create PID file: {0}")]
    CreatePidFileError(String),
    #[error("Fail to read PID file: {0}")]
    ReadPidFileError(String),
    #[error("Fail to write PID file: {0}")]
    WritePidFileError(String),
    #[error("Fail to parse content of PID file : {0}")]
    ParsePidFileContentError(String),
    #[error("Fail to delete PID file: {0}")]
    DeletePidFileError(String),
}
