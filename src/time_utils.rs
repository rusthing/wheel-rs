//! # 时间工具
use std::time::SystemTime;

/// # 获取当前时间戳（毫秒）
///
/// 该函数返回自 Unix 纪元（1970年1月1日 00:00:00 UTC）以来的毫秒数。
/// 可用于生成唯一标识符、记录事件时间戳或进行时间相关的计算。
///
/// ## 返回值
///
/// 返回一个 u128 类型的毫秒时间戳。
///
/// ## Panics
///
/// 当系统时间早于 Unix 纪元时间时，函数会 panic。
/// 在正常运行的系统中，这种情况几乎不会发生。
///
/// ## 示例
///
/// ```
/// use your_crate::utils::time_utils::get_current_timestamp;
///
/// let timestamp = get_current_timestamp();
/// println!("当前时间戳: {}", timestamp);
/// ```
pub fn get_current_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
