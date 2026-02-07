//! PID 文件操作相关的错误类型定义
//!
//! 该模块定义了处理 PID 文件时可能遇到的各种错误类型，
//! 包括路径无效、文件操作失败、内容解析错误等情况。

use std::path::PathBuf;
use thiserror::Error;

/// PID 文件操作错误枚举
///
/// 定义了 PID 文件处理过程中可能出现的所有错误类型
#[derive(Error, Debug)]
pub enum PidError {
    /// PID 文件路径无效错误
    ///
    /// 当提供的 PID 文件路径不符合要求时返回此错误
    #[error("Invalid PID file path: {0}")]
    InvalidPidFilePath(PathBuf),

    /// 打开 PID 文件失败错误
    ///
    /// 当无法打开指定的 PID 文件时返回此错误
    #[error("Fail to open PID file: {0}")]
    OpenPidFile(String),

    /// 创建 PID 文件失败错误
    ///
    /// 当无法创建新的 PID 文件时返回此错误
    #[error("Fail to create PID file: {0}")]
    CreatePidFile(String),

    /// 读取 PID 文件失败错误
    ///
    /// 当无法读取 PID 文件内容时返回此错误
    #[error("Fail to read PID file: {0}")]
    ReadPidFile(String),

    /// 写入 PID 文件失败错误
    ///
    /// 当无法向 PID 文件写入数据时返回此错误
    #[error("Fail to write PID file: {0}")]
    WritePidFile(String),

    /// 解析 PID 文件内容失败错误
    ///
    /// 当 PID 文件内容格式不正确或无法解析时返回此错误
    #[error("Fail to parse content of PID file : {0}")]
    ParsePidFileContent(String),

    /// 删除 PID 文件失败错误
    ///
    /// 当无法删除指定的 PID 文件时返回此错误
    #[error("Fail to delete PID file: {0}")]
    DeletePidFile(String),
}
