use std::process::Child;

/// 检查进程是否还活着
pub fn is_process_alive(child: &mut Child) -> bool {
    match child.try_wait() {
        Ok(Some(_)) => false, // 进程已退出
        Ok(None) => true,     // 进程仍在运行
        Err(_) => false,      // 检查失败，认为已死亡
    }
}

/// 杀死进程
pub fn kill_process(mut child: Child) -> Result<(), Box<dyn std::error::Error>> {
    child.kill()?;
    child.wait()?;
    Ok(())
}
