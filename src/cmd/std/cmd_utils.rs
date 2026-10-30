//! # 命令行工具函数模块
//!
//! 提供执行外部命令和管理进程的实用工具函数。
//!
//! 该模块包含以下主要功能：
//! - 执行外部命令并获取输出
//! - 检查进程是否存活
//! - 杀死进程

use crate::cmd::cmd_error::CmdError;
use std::process::{Child, Command};
use tracing::debug;

/// # 执行外部命令
///
/// 执行指定的外部命令并返回其标准输出。此函数会等待命令执行完成，
/// 并检查命令执行结果状态。
///
/// ## 参数
///
/// * `cmd` - 要执行的命令名称
/// * `args` - 命令参数切片
///
/// ## 返回值
///
/// 返回命令的标准输出字节向量，或者包含错误信息的 [CmdError]。
///
/// ## 错误处理
///
/// 如果命令执行失败或返回非零退出码，则返回相应的 [CmdError]。
///
/// ## 示例
///
/// ```
/// use wheel_rs::cmd::std::cmd_utils::execute;
///
/// let output = execute("echo", &["Hello, world!"]);
/// match output {
///     Ok(bytes) => {
///         let output_str = String::from_utf8_lossy(&bytes);
///         println!("Command output: {}", output_str);
///     }
///     Err(e) => eprintln!("Command failed: {:#}", e),
/// }
/// ```
pub fn execute(cmd: &str, args: &[&str]) -> Result<Vec<u8>, CmdError> {
    debug!("executing command: {} {}", cmd, args.join(" "));
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(CmdError::Execute)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(CmdError::Run(if stderr.is_empty() {
            // 命令把错误信息写到了 stdout 或者什么都没写，至少把退出码带出去
            format!("command `{cmd}` exited with {}", output.status)
        } else {
            stderr
        }));
    }

    Ok(output.stdout)
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
pub fn is_process_alive(child: &mut Child) -> bool {
    debug!("checking if process is alive: {}", child.id());
    match child.try_wait() {
        Ok(Some(_)) => false, // 进程已退出
        Ok(None) => true,     // 进程仍在运行
        Err(_) => false,      // 检查失败，认为已死亡
    }
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
/// 如果杀死进程过程中发生错误，则返回相应的错误信息。
pub fn kill_process(mut child: Child) -> Result<(), Box<dyn std::error::Error>> {
    debug!("killing process: {}", child.id());
    child.kill()?;
    child.wait()?;
    Ok(())
}

/// # 根据进程ID杀死进程
///
/// 根据指定的进程ID强制终止相应进程。
///
/// ## 参数
///
/// * `process` - 要杀死的进程ID
///
/// ## 返回值
///
/// 如果成功杀死进程则返回 `Ok(())`，否则返回包含错误信息的 Result。
///
/// ## 平台差异
///
/// 本库依赖 `nix`，目前只在 Unix 上可用：
/// - Unix系统: 使用 `nix` crate 的 `kill` 函数发送 `SIGKILL` 信号
/// - Windows系统: 保留 `taskkill /F /PID <process>` 分支，但整库尚不支持 Windows
///
/// ## 错误处理
///
/// 如果杀死进程过程中发生错误，则返回相应的错误信息。
pub fn kill_process_by_id(pid: u32) -> std::io::Result<()> {
    debug!("killing process by id: {}", pid);
    #[cfg(unix)]
    {
        // 使用 nix crate 发送 SIGKILL 信号，避免通过 shell 命令
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(pid as i32),
            nix::sys::signal::Signal::SIGKILL,
        )?;
    }

    #[cfg(windows)]
    {
        Command::new("taskkill")
            .arg("/F")
            .arg("/PID")
            .arg(&pid.to_string())
            .output()?;
    }

    Ok(())
}
