//! # 时间工具
//!
//! 提供时间相关的实用工具函数。

use chrono::Utc;
use std::time::Duration;
use tokio::time::{interval, MissedTickBehavior};

/// # 获取当前 Unix 时间戳（秒）
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

/// # 获取当前 Unix 时间戳（毫秒）
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

/// # 获取当前 Unix 时间戳（纳秒）。
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

/// # 将 `Duration` 转为人类可读的字符串
///
/// 使用 `humantime` 格式化，输出如 `"5s"`、`"10m"`、`"1h 30m"` 等格式，
/// 适用于存入数据库或日志展示。
pub fn duration_to_string(d: Duration) -> String {
    humantime::format_duration(d).to_string()
}

/// # 将人类可读的时长字符串解析为 `Duration`
///
/// 支持 `humantime` 格式，如 `"5s"`、`"10m"`、`"1h 30m"` 等。
/// 解析失败时返回零时长（`Duration::default()`）。
pub fn string_to_duration(s: String) -> Duration {
    humantime::parse_duration(&s).unwrap_or_default()
}

/// # 构建定时器
///
/// 创建一个 `tokio` 定时器，错过的时间点会被跳过（`MissedTickBehavior::Skip`），
/// 适用于周期性任务调度场景。
///
/// ## 参数
///
/// * `period` - 定时器触发间隔
///
/// ## 返回值
///
/// 返回配置好的 [`tokio::time::Interval`]。
pub fn build_ticker(period: Duration) -> tokio::time::Interval {
    let mut ticker = interval(period);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip); // 忽略错过的时间点
    ticker
}