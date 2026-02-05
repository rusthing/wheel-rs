use crate::process::SignalError;
use crate::process::SignalError::{InvalidInstructionError, SendSignalError};
use log::{debug, info};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use tokio::signal::unix::{signal, SignalKind};

/// # 通过指令发送系统信号给指定进程
///
/// 根据信号字符串向目标进程发送相应的系统信号，支持多种常用信号。
///
/// ## 参数
///
/// * `signal_str` - 信号名称字符串，如 "stop", "reload", "quit", "kill" 等
/// * `pid` - 进程ID，指定要发送信号的目标进程
///
/// ## 返回值
///
/// * `Ok(())` - 信号发送成功
/// * `Err(Box<dyn std::error::Error>)` - 信号发送失败
///
/// ## 支持的指令
///
/// * `hangup` - 挂起进程，发送`SIGHUP`信号(kill -1)，用于暂停程序运行
/// * `continue` - 继续运行进程，发送`SIGCONT`信号(kill -18)，用于恢复程序运行
/// * `interrupt` - 发送`SIGINT`信号(kill -2)，用于中断程序运行
/// * `quit` - 发送`SIGQUIT`信号(kill -3)，用于退出程序，但保存进程运行状态
/// * `stop`|`terminate` - 发送`SIGTERM`信号 (kill -15)，用于终止程序，优雅退出
/// * `kill` - 发送`SIGKILL`信号(kill -9)，用于强制终止程序(顺带删除PID文件)
///
/// ## 错误处理
///
/// 当指定的信号名称无效时，函数会返回错误
pub fn send_signal_by_instruction(instruction: &str, pid: i32) -> Result<(), SignalError> {
    debug!("send signal by {instruction} instruction -> {pid}");
    let instruction = instruction.to_lowercase();
    let signal = match instruction.as_str() {
        "hangup" => Signal::SIGHUP,
        "cont" => Signal::SIGCONT,
        "interrupt" => Signal::SIGINT,
        "stop" | "terminate" => Signal::SIGTERM,
        "quit" => Signal::SIGQUIT,
        "kill" => Signal::SIGKILL,
        _ => Err(InvalidInstructionError(instruction.to_string()))?,
    };
    kill(Pid::from_raw(pid), signal).map_err(|_| SendSignalError(signal.to_string()))
}

/// # 异步监听系统信号
///
/// 该函数异步等待系统信号的到来，目前为空实现，可用于扩展信号处理功能。
pub fn watch_signal() {
    tokio::spawn(async move {
        debug!("watching signal...");
        let mut sighup_stream =
            signal(SignalKind::hangup()).expect("Failed to register signal handler: SIGHUP");
        let mut sigcont_stream =
            signal(SignalKind::from_raw(18)).expect("Failed to register signal handler: SIGCONT");
        let mut sigint_stream =
            signal(SignalKind::interrupt()).expect("Failed to register signal handler: SIGINT");
        let mut sigquit_stream =
            signal(SignalKind::quit()).expect("Failed to register signal handler: SIGQUIT");
        let mut sigterm_stream =
            signal(SignalKind::terminate()).expect("Failed to register signal handler: SIGTERM");

        loop {
            tokio::select! {
                _ = sighup_stream.recv() => {
                    info!("程序挂起(SIGHUP)");
                }
                _ = sigcont_stream.recv() => {
                    info!("程序继续运行(SIGCONT)");
                }
                _ = sigint_stream.recv() => {
                    info!("程序中断运行(SIGINT)");
                    break;
                }
                _ = sigquit_stream.recv() => {
                    info!("程序退出运行(SIGQUIT)");
                    break;
                }
                _ = sigterm_stream.recv() => {
                    info!("程序终止运行(SIGTERM)");
                    break;
                }
            }
        }
    });
}
