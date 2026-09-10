//! # 时间工具
//!
//! 提供时间相关的实用工具函数。

use std::time::{SystemTime, SystemTimeError};

/// # 获取当前时间戳（毫秒）
///
/// 返回自 Unix 纪元（1970-01-01 00:00:00 UTC）以来的毫秒数，
/// 可用于生成唯一标识符、记录事件时间或进行时间相关的计算。
///
/// ## 错误
///
/// 当系统时间早于 Unix 纪元时返回 `SystemTimeError`；在正常运行的系统上几乎不会发生。
///
/// ## 示例
///
/// ```
/// use wheel_rs::time_utils::now_ts;
///
/// let timestamp = now_ts().unwrap();
/// println!("当前毫秒时间戳: {timestamp}");
/// ```
pub fn now_ts() -> Result<u64, SystemTimeError> {
    Ok(SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as u64)
}

/// # 获取当前时间戳（纳秒）
///
/// 返回自 Unix 纪元（1970-01-01 00:00:00 UTC）以来的纳秒数，
/// 可用于需要更高时间精度的场景，如性能测量或生成唯一标识符。
///
/// 返回一个 `u64` 类型的纳秒时间戳。
/// ## 错误
///
/// 当系统时间早于 Unix 纪元时返回 `SystemTimeError`；在正常运行的系统上几乎不会发生。
///
/// ## 示例
///
/// ```
/// use wheel_rs::time_utils::now_ns;
///
/// let timestamp = now_ns().unwrap();
/// println!("当前纳秒时间戳: {timestamp}");
/// ```
pub fn now_ns() -> Result<u64, SystemTimeError> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_nanos() as u64)
}
