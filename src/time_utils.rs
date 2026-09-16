//! # 时间工具
//!
//! 提供时间相关的实用工具函数。

use chrono::Utc;

/// 获取当前 Unix 时间戳（秒）。
///
/// 返回自 Unix 纪元（1970-01-01 00:00:00 UTC）以来的秒数。
/// 适用于生成唯一标识符、记录事件时间等常规场景。
///
/// ## 示例
///
/// ```
/// use wheel_rs::time_utils::now_ts;
///
/// let timestamp = now_ts();
/// println!("当前秒级时间戳: {timestamp}");
/// ```
pub fn now_ts() -> u64 {
    Utc::now().timestamp() as u64
}

/// 获取当前 Unix 时间戳（毫秒）。
///
/// 返回自 Unix 纪元（1970-01-01 00:00:00 UTC）以来的毫秒数。
/// 精度介于秒和纳秒之间，适用于需要毫秒级精度的一般场景。
///
/// ## 示例
///
/// ```
/// use wheel_rs::time_utils::now_ms;
///
/// let timestamp = now_ms();
/// println!("当前毫秒时间戳: {timestamp}");
/// ```
pub fn now_ms() -> u64 {
    Utc::now().timestamp_millis() as u64
}

/// 获取当前 Unix 时间戳（纳秒）。
///
/// 返回自 Unix 纪元（1970-01-01 00:00:00 UTC）以来的纳秒数。
/// 适用于需要更高时间精度的场景，如性能测量或分布式唯一 ID 生成。
///
/// ## 示例
///
/// ```
/// use wheel_rs::time_utils::now_ns;
///
/// let timestamp = now_ns();
/// println!("当前纳秒时间戳: {timestamp}");
/// ```
pub fn now_ns() -> u64 {
    Utc::now().timestamp_nanos_opt().unwrap() as u64
}