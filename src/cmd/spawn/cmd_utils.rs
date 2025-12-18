//! # 命令行工具函数模块
//!
//! 提供执行外部命令和管理进程的实用工具函数。
//!
//! 该模块包含以下主要功能：
//! - 执行外部命令并获取输出
//! - 检查进程是否存活
//! - 杀死进程
use crate::cmd::cmd_error::CmdError;
use bytes::Bytes;
use log::{debug, error, warn};
use std::process::Stdio;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::broadcast::Sender;
use tokio::sync::oneshot;

/// # 执行外部命令进程
///
/// 执行指定的外部命令进程并返回其子进程句柄。注意：调用此函数后，
/// `Child` 实例的所有权将转移给调用者，同时 `Child.stdout` 的所有权
/// 会被移动用于异步读取。
///
/// ## 参数
///
/// * `cmd` - 要执行的命令名称
/// * `args` - 命令参数切片
/// * `data_sender` - 用于发送命令输出数据的广播发送者
/// * `process_exit_sender` - 用于发送进程结束信号的通道发送者
/// * `read_buffer_size` - 可选的读取缓冲区大小
///
/// ## 返回值
///
/// 返回命令的子进程句柄，或者包含错误信息的 [CmdError]。
///
/// ## 示例
///
/// ```rust
/// use wheel_rs::cmd::spawn::cmd_utils::execute;
/// use tokio::sync::broadcast;
/// use std::sync::mpsc;
///
/// let (data_sender, _) = broadcast::channel(100);
/// let (process_exit_sender, _) = mpsc::channel();
/// let child = execute("ls", &["-l"], data_sender, process_exit_sender, None);
/// ```
pub fn execute(
    cmd: &str,
    args: &[&str],
    data_sender: Sender<Bytes>,
    process_exit_sender: oneshot::Sender<()>,
    read_buffer_size: Option<usize>,
) -> Result<Child, CmdError> {
    debug!("command execute start: {} {}", cmd, args.join(" "));
    let mut child = Command::new(cmd) // 创建新的命令实例
        .args(args) // 添加命令参数
        .stdout(Stdio::piped()) // 将标准输出重定向到管道，以便父进程可以读取
        .stderr(Stdio::null()) // 丢弃标准错误输出
        .spawn() // 启动命令并返回子进程句柄
        .map_err(|e| CmdError::ExecuteFail(e))?; // 将可能的错误转换为CmdError类型
    debug!("command execute started: {}", cmd);
    // 获取标准输出
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| CmdError::TakeStdoutError("command process stdout not piped".to_string()))?;

    // 异步读取输出
    tokio::spawn(read_stdout(
        stdout,
        data_sender,
        process_exit_sender,
        read_buffer_size,
    ));

    Ok(child)
}

/// # 读取子进程的输出
///
/// 异步读取子进程的输出并转发给指定的发送者。
///
/// ## 参数
///
/// * `stdout` - 子进程的输出流
/// * `data_sender` - 用于转发输出数据的广播发送者
/// * `process_exit_sender` - 用于发送进程结束信号的通道发送者
/// * `read_buffer_size` - 可选的读取缓冲区大小
///
/// ## 返回值
///
/// 无返回值，因为该函数是异步的。
async fn read_stdout(
    stdout: ChildStdout,
    data_sender: Sender<Bytes>,
    process_exit_sender: oneshot::Sender<()>,
    read_buffer_size: Option<usize>,
) {
    let mut reader = BufReader::new(stdout);
    let mut buffer = vec![0u8; read_buffer_size.unwrap_or(65536)];
    loop {
        match reader.read(&mut buffer).await {
            Ok(0) => {
                debug!("command process stdout closed");
                break;
            }
            Ok(n) => {
                // 有订阅者才发送消息
                let receiver_count = data_sender.receiver_count();
                if receiver_count > 0 {
                    debug!("command process receiver count: {}", receiver_count);
                    let data = Bytes::copy_from_slice(&buffer[..n]);
                    let _ = data_sender.send(data).map_err(|e| {
                        warn!("Failed to send command process output to receiver: {}", e)
                    });
                }
            }
            Err(e) => {
                error!("read command process stdout error: {}", e);
                break;
            }
        }
    }
    let _ = process_exit_sender.send(());
}

/// # 检查进程是否还活着
///
/// 检查指定的子进程是否仍在运行。此函数不会阻塞，也不会消耗进程资源。
///
/// ## 参数
///
/// * `child` - 要检查的子进程可变引用
///
/// ## 返回值
///
/// 如果进程仍在运行则返回 `true`，否则返回 `false`。
///
/// ## 错误处理
///
/// 如果无法获取进程ID，则返回 [CmdError::EmptyId] 错误。
pub fn is_process_alive(child: &mut Child) -> Result<bool, CmdError> {
    debug!(
        "checking if process is alive: {}",
        child.id().ok_or_else(|| CmdError::EmptyId)?
    );
    Ok(match child.try_wait() {
        Ok(Some(_)) => false, // 进程已退出
        Ok(None) => true,     // 进程仍在运行
        Err(_) => false,      // 检查失败，认为已死亡
    })
}

/// # 杀死进程
///
/// 强制终止指定的子进程并等待其完全退出。调用此函数会获取 `Child` 实例
/// 的所有权，并在完成后释放该资源。
///
/// ## 参数
///
/// * `child` - 要杀死的子进程
///
/// ## 返回值
///
/// 如果成功杀死进程则返回 `Ok(())`，否则返回包含错误信息的 Result。
///
/// ## 错误处理
///
/// 如果杀死进程过程中发生错误，则返回 [CmdError::KillFail] 错误。
pub async fn kill_process(mut child: Child) -> Result<(), CmdError> {
    debug!(
        "killing process: {}",
        child.id().ok_or_else(|| CmdError::EmptyId)?
    );
    Ok(child.kill().await.map_err(|e| {
        error!("kill process fail: {}", e);
        CmdError::KillFail(e)
    })?)
}
