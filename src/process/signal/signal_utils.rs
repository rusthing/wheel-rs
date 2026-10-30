//! # 信号处理工具函数
//!
//! 提供系统信号的发送和监听功能，支持常见的Unix信号处理。
//! 包括通过指令发送信号、异步信号监听等功能。

use crate::process::SignalError;
use libc::pid_t;
use nix::sys::signal::kill;
use nix::unistd::Pid;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use tracing::{debug, error, info};

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
/// 当指定的信号名称无效时，函数会返回 `SignalError::InvalidInstruction`。
/// 若信号发送失败（如权限不足或进程不存在），则返回 `SignalError::SendSignal`。
pub fn send_signal_by_instruction(instruction: &str, pid: u32) -> Result<(), SignalError> {
    debug!("send signal by {instruction} instruction -> {pid}");
    let instruction = instruction.to_lowercase();
    let signal = match instruction.as_str() {
        "hangup" => nix::sys::signal::Signal::SIGHUP,
        "cont" => nix::sys::signal::Signal::SIGCONT,
        "interrupt" => nix::sys::signal::Signal::SIGINT,
        "stop" | "terminate" => nix::sys::signal::Signal::SIGTERM,
        "quit" => nix::sys::signal::Signal::SIGQUIT,
        "kill" => nix::sys::signal::Signal::SIGKILL,
        _ => return Err(SignalError::InvalidInstruction(instruction)),
    };
    kill(Pid::from_raw(pid as pid_t), signal)
        .map_err(|e| SignalError::SendSignal(format!("{signal}: {e}")))
}

/// # 异步监听系统信号
///
/// 该函数异步监听多种系统信号（如 `SIGHUP`、`SIGINT`、`SIGTERM` 等），并在接收到信号时
/// 广播给调用方。监听任务由 `tokio::spawn` 启动，**不会 panic**：注册失败或广播失败
/// 只记录日志。
///
/// ## 监听的信号
///
/// * `SIGHUP` - 程序挂起信号，记录日志但不退出。
/// * `SIGCONT` - 程序继续运行信号，记录日志但不退出。
/// * `SIGINT` - 程序中断信号（如 Ctrl+C），记录日志并退出监听循环。
/// * `SIGTERM` - 程序终止信号，记录日志并退出监听循环。
/// * `SIGQUIT` - 程序退出信号，记录日志并退出监听循环。
///
/// ## 返回值
///
/// 返回一个 `tokio::sync::broadcast::Receiver`，调用方可通过 `recv()` 异步接收监听到的系统信号
/// （类型为 `nix::sys::signal::Signal`）。注意：**所有** `Receiver` 都被丢弃后，后续信号
/// 只记录日志，不会导致监听任务退出。
///
/// ## 注意事项
///
/// - 该函数使用 `tokio::spawn` 启动异步任务，需在 `tokio` 运行时环境中调用。
/// - 信号处理逻辑目前仅为日志输出 + 广播，可根据实际需求扩展具体业务逻辑。
pub fn watch_signal() -> Receiver<nix::sys::signal::Signal> {
    let (sender, receiver) = broadcast::channel(16);
    tokio::spawn(async move {
        if let Err(e) = watch_signal_internal(sender).await {
            error!("watch signal stopped: {e}");
        }
    });
    receiver
}

async fn watch_signal_internal(
    sender: Sender<nix::sys::signal::Signal>,
) -> Result<(), SignalError> {
    debug!("watching signal...");
    let mut sighup_stream = signal(SignalKind::hangup())
        .map_err(|_| SignalError::RegisterSignalHandler("SIGHUP".to_string()))?;
    // 使用 libc::SIGCONT 提高可移植性，避免硬编码信号号 18
    let mut sigcont_stream = signal(SignalKind::from_raw(libc::SIGCONT))
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
                broadcast_signal(&sender, signal);
                info!("程序挂起({signal})");
            }
            _ = sigcont_stream.recv() => {
                let signal = nix::sys::signal::Signal::SIGCONT;
                broadcast_signal(&sender, signal);
                info!("程序继续运行({signal})");
            }
            _ = sigint_stream.recv() => {
                let signal = nix::sys::signal::Signal::SIGINT;
                broadcast_signal(&sender, signal);
                info!("程序中断运行({signal})");
                break;
            }
            _ = sigquit_stream.recv() => {
                let signal = nix::sys::signal::Signal::SIGQUIT;
                broadcast_signal(&sender, signal);
                info!("程序退出运行({signal})");
                break;
            }
            _ = sigterm_stream.recv() => {
                let signal = nix::sys::signal::Signal::SIGTERM;
                broadcast_signal(&sender, signal);
                info!("程序终止运行({signal})");
                break;
            }
        }
    }
    Ok(())
}

/// 广播一次信号。
///
/// `broadcast::Sender::send` 在**没有存活订阅者**时返回 `Err`，这是完全正常的状态
/// （调用方可能还没开始 `recv`，或者已经 drop 掉 Receiver），不能当作错误 panic，
/// 否则整个监听任务会在毫无征兆的情况下挂掉。
fn broadcast_signal(sender: &Sender<nix::sys::signal::Signal>, signal: nix::sys::signal::Signal) {
    if sender.send(signal).is_err() {
        debug!("no alive signal receiver, drop {signal}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_instruction_is_reported() {
        assert!(matches!(
            send_signal_by_instruction("no-such-signal", std::process::id()),
            Err(SignalError::InvalidInstruction(_))
        ));
    }

    #[test]
    fn instruction_is_case_insensitive() {
        // "CONT" 发给自身进程不会造成副作用，只验证大小写处理与发送链路可用
        assert!(send_signal_by_instruction("CONT", std::process::id()).is_ok());
    }
}
