//! # 命令行工具函数模块
//!
//! 提供执行外部命令和管理进程的实用工具函数。
//!
//! 该模块包含以下主要功能：
//! - 执行外部命令并获取输出
//! - 检查进程是否存活
//! - 杀死进程

use crate::cmd::cmd_error::CmdError;
use log::debug;
use std::process::{Child, Command, Stdio};

/// # 执行外部命令
///
/// 执行指定的外部命令并返回其标准输出。
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
/// ## 示例
///
/// ```
/// use wheel_rs::cmd::cmd_utils::exec;
///
/// let output = exec("echo", &["Hello, world!"]);
/// match output {
///     Ok(bytes) => {
///         let output_str = String::from_utf8_lossy(&bytes);
///         println!("Command output: {}", output_str);
///     }
///     Err(e) => eprintln!("Command failed: {}", e),
/// }
/// ```
pub fn exec(cmd: &str, args: &[&str]) -> Result<Vec<u8>, CmdError> {
    debug!("executing command: {} {}", cmd, args.join(" "));
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| CmdError::ExecuteFail(e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(CmdError::RunFail(stderr));
    }

    Ok(output.stdout)
}

/// # 启动外部命令进程
///
/// 启动指定的外部命令进程并返回其子进程句柄。
///
/// ## 参数
///
/// * `cmd` - 要执行的命令名称
/// * `args` - 命令参数切片
///
/// ## 返回值
///
/// 返回命令的子进程句柄，或者包含错误信息的 [CmdError]。
///
/// ## 示例
///
/// ```
/// use wheel_rs::cmd::cmd_utils::spawn_cmd;
///
/// let child = spawn_cmd("sleep", &["5"]);
/// match child {
///     Ok(mut child) => {
///         // 可以对子进程进行操作
///         // 记得处理子进程资源
///         let _ = child.wait();
///     }
///     Err(e) => eprintln!("Failed to spawn command: {}", e),
/// }
/// ```
pub fn spawn_cmd(cmd: &str, args: &[&str]) -> Result<tokio::process::Child, CmdError> {
    debug!("spawn command: {} {}", cmd, args.join(" "));
    Ok(tokio::process::Command::new(cmd) // 创建新的命令实例
        .args(args) // 添加命令参数
        .stdout(Stdio::piped()) // 将标准输出重定向到管道，以便父进程可以读取
        .stderr(Stdio::null()) // 丢弃标准错误输出
        .spawn() // 启动命令并返回子进程句柄
        .map_err(|e| CmdError::ExecuteFail(e))?) // 将可能的错误转换为CmdError类型
}

/// # 检查进程是否还活着
///
/// 检查指定的子进程是否仍在运行。
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
/// 强制终止指定的子进程并等待其完全退出。
///
/// ## 参数
///
/// * `child` - 要杀死的子进程
///
/// ## 返回值
///
/// 如果成功杀死进程则返回 `Ok(())`，否则返回包含错误信息的 Result。
pub fn kill_process(mut child: Child) -> Result<(), Box<dyn std::error::Error>> {
    debug!("killing process: {}", child.id());
    child.kill()?;
    child.wait()?;
    Ok(())
}
