//! # 信号处理工具函数
//!
//! 提供系统信号的发送和监听功能，支持常见的Unix信号处理。
//! 包括通过指令发送信号、异步信号监听等功能。

use crate::process::SignalError;
use log::{debug, info};
use nix::sys::signal::kill;
use nix::unistd::Pid;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

/// # 通过指令发送系统信号给指定进程
///
/// 根据信号字符串向目标进程发送相应的系统信号，支持多种常用信号。
///
/// ## 参数
///
/// * `instruction` - 信号名称字符串，如 `"hangup"`, `"stop"`, `"kill"` 等。
/// * `pid` - 进程ID，指定要发送信号的目标进程。
///
/// ## 返回值
///
/// * `Ok(())` - 信号发送成功。
/// * `Err(SignalError)` - 信号发送失败或指令无效。
///
/// ## 支持的指令
///
/// * `"hangup"` - 发送 `SIGHUP` 信号 (`kill -1`)，用于挂起进程。
/// * `"cont"` - 发送 `SIGCONT` 信号 (`kill -18`)，用于继续运行进程。
/// * `"interrupt"` - 发送 `SIGINT` 信号 (`kill -2`)，用于中断程序运行。
/// * `"stop"` / `"terminate"` - 发送 `SIGTERM` 信号 (`kill -15`)，用于优雅终止程序。
/// * `"quit"` - 发送 `SIGQUIT` 信号 (`kill -3`)，用于退出程序并生成核心转储。
/// * `"kill"` - 发送 `SIGKILL` 信号 (`kill -9`)，用于强制终止程序。
///
/// ## 错误处理
///
/// 当指定的信号名称无效时，函数会返回 `InvalidInstructionError`。
/// 若信号发送失败（如权限不足或进程不存在），则返回 `SendSignalError`。
pub fn send_signal_by_instruction(instruction: &str, pid: i32) -> Result<(), SignalError> {
    debug!("send signal by {instruction} instruction -> {pid}");
    let instruction = instruction.to_lowercase();
    let signal = match instruction.as_str() {
        "hangup" => nix::sys::signal::Signal::SIGHUP,
        "cont" => nix::sys::signal::Signal::SIGCONT,
        "interrupt" => nix::sys::signal::Signal::SIGINT,
        "stop" | "terminate" => nix::sys::signal::Signal::SIGTERM,
        "quit" => nix::sys::signal::Signal::SIGQUIT,
        "kill" => nix::sys::signal::Signal::SIGKILL,
        _ => Err(SignalError::InvalidInstruction(instruction.to_string()))?,
    };
    kill(Pid::from_raw(pid), signal).map_err(|_| SignalError::SendSignal(signal.to_string()))
}

/// # 异步监听系统信号
///
/// 该函数异步监听多种系统信号（如 `SIGHUP`、`SIGINT`、`SIGTERM` 等），并在接收到信号时执行相应操作。
/// 目前实现了基本的日志输出功能，未来可根据需求扩展更多信号处理逻辑。
///
/// ## 监听的信号
///
/// * `SIGHUP` - 程序挂起信号，记录日志但不退出。
/// * `SIGCONT` - 程序继续运行信号，记录日志但不退出。
/// * `SIGINT` - 程序中断信号（如 Ctrl+C），记录日志并退出监听循环。
/// * `SIGTERM` - 程序终止信号，记录日志并退出监听循环。
/// * `SIGQUIT` - 程序退出信号，记录日志并退出监听循环。
///
/// ## 注意事项
///
/// - 该函数使用 `tokio::spawn` 启动异步任务，需在 `tokio` 运行时环境中调用。
/// - 信号处理逻辑目前仅为日志输出，可根据实际需求扩展具体业务逻辑。
pub fn watch_signal() -> Receiver<nix::sys::signal::Signal> {
    let (sender, receiver) = broadcast::channel(16);
    tokio::spawn(async {
        watch_signal_internal(sender)
            .await
            .expect("watch signal error");
    });
    receiver
}

async fn watch_signal_internal(
    sender: Sender<nix::sys::signal::Signal>,
) -> Result<(), SignalError> {
    debug!("watching signal...");
    let mut sighup_stream = signal(SignalKind::hangup())
        .map_err(|_| SignalError::RegisterSignalHandler("SIGHUP".to_string()))?;
    let mut sigcont_stream = signal(SignalKind::from_raw(18))
        .map_err(|_| SignalError::RegisterSignalHandler("SIGCONT".to_string()))?;
    let mut sigint_stream = signal(SignalKind::interrupt())
        .map_err(|_| SignalError::RegisterSignalHandler("SIGINT".to_string()))?;
    let mut sigquit_stream = signal(SignalKind::quit())
        .map_err(|_| SignalError::RegisterSignalHandler("SIGQUIT".to_string()))?;
    let mut sigterm_stream = signal(SignalKind::terminate())
        .map_err(|_| SignalError::RegisterSignalHandler("SIGTERM".to_string()))?;

    loop {
        tokio::select! {
            _ = sighup_stream.recv() => {
                let signal = nix::sys::signal::Signal::SIGHUP;
                sender.send(signal).expect(format!("send signal error: {signal}").as_str());
                info!("程序挂起({signal})");
            }
            _ = sigcont_stream.recv() => {
                let signal = nix::sys::signal::Signal::SIGCONT;
                sender.send(signal).expect(format!("send signal error: {signal}").as_str());
                info!("程序继续运行({signal})");
            }
            _ = sigint_stream.recv() => {
                let signal = nix::sys::signal::Signal::SIGINT;
                sender.send(signal).expect(format!("send signal error: {signal}").as_str());
                info!("程序中断运行({signal})");
                break;
            }
            _ = sigquit_stream.recv() => {
                let signal = nix::sys::signal::Signal::SIGQUIT;
                sender.send(signal).expect(format!("send signal error: {signal}").as_str());
                info!("程序退出运行({signal})");
                break;
            }
            _ = sigterm_stream.recv() => {
                let signal = nix::sys::signal::Signal::SIGTERM;
                sender.send(signal).expect(format!("send signal error: {signal}").as_str());
                info!("程序终止运行({signal})");
                break;
            }
        }
    }
    Ok(())
}
