//! # 时间工具
//!
//! 提供时间相关的实用工具函数：当前时间戳、时长字符串互转、定时器构建，以及
//! 外部时间戳的规范化校验（[`normalize_ms`]）。

use chrono::Utc;
use std::time::Duration;
use tokio::time::{MissedTickBehavior, interval};
use tracing::warn;

/// 可信毫秒时间戳的下界：2020-01-01T00:00:00Z
///
/// 小于它的值不可能是「合理的毫秒时间戳」：秒级时间戳（10 位）、毫秒级但明显
/// 是错误默认值（`0` / `1`）都落在下面。
pub const MIN_PLAUSIBLE_MS: u64 = 1_577_836_800_000;

/// 可信毫秒时间戳允许的超前偏差（毫秒，一天）
///
/// 对端时钟可能比本机快，但快过一天说明配置错了或时钟彻底跑飞。留一天既容忍
/// 时区/校时误差，也挡住「把纳秒当毫秒」这类量级错误。
pub const MAX_FUTURE_SKEW_MS: u64 = 24 * 60 * 60 * 1000;

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
    // 系统时钟早于 1970 时返回 0，而不是把负数 as 成巨大的 u64
    Utc::now().timestamp().max(0) as u64
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
    Utc::now().timestamp_millis().max(0) as u64
}

/// # 获取当前 Unix 时间戳（纳秒）
///
/// 返回自 Unix 纪元（1970-01-01 00:00:00 UTC）以来的纳秒数。
/// 适用于需要更高时间精度的场景，如性能测量或分布式唯一 ID 生成。
///
/// 纳秒时间戳的可表示范围约为 1677 年 ~ 2262 年，超出该范围或系统时钟早于
/// 1970 年时返回 `0`（不 panic）。
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
    Utc::now().timestamp_nanos_opt().unwrap_or_default().max(0) as u64
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
///
/// 参数取 `&str`，调用方无需交出 `String` 所有权；需要 `String` 时传 `&s` 即可。
///
/// ## 示例
///
/// ```
/// use std::time::Duration;
/// use wheel_rs::time_utils::string_to_duration;
///
/// assert_eq!(string_to_duration("5s"), Duration::from_secs(5));
/// // 解析失败不 panic，返回零时长
/// assert_eq!(string_to_duration("not-a-duration"), Duration::ZERO);
/// ```
pub fn string_to_duration(s: &str) -> Duration {
    humantime::parse_duration(s).unwrap_or_default()
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

/// # 这个值像不像一个可信的毫秒时间戳
///
/// 判据只有量级：落在 `[`[`MIN_PLAUSIBLE_MS`]`, 当前时间 + `[`MAX_FUTURE_SKEW_MS`]`]`
/// 之内才算数。**不做秒→毫秒的换算**——猜错单位比取当前时间更糟（10 位数字既可
/// 能是秒，也可能是某个错误的默认值）。
///
/// ## 示例
///
/// ```
/// use wheel_rs::time_utils::{is_plausible_ms, now_ms};
///
/// assert!(is_plausible_ms(now_ms()));
/// // 秒级时间戳：量级不对
/// assert!(!is_plausible_ms(1_800_000_000));
/// assert!(!is_plausible_ms(0));
/// ```
pub fn is_plausible_ms(ms: u64) -> bool {
    let max = now_ms().saturating_add(MAX_FUTURE_SKEW_MS);
    (MIN_PLAUSIBLE_MS..=max).contains(&ms)
}

/// # 把外部给的时间戳规整成可信的毫秒时间戳
///
/// 外部输入（设备上报、第三方回调、手工填的配置）的时间戳是不可信的：可能缺失、
/// 可能是 `0`、也可能是秒级而不是毫秒级。这个值一旦被当成毫秒用，后果不是「显示
/// 难看」而是逻辑错乱：
///
/// - 秒级当毫秒用：`now - ts` 变成十几位的正数，任何「是否在窗口内」的判断都判
///   过期，本该合并的事件每次都被当成新事件；
/// - `0` 当毫秒用：参与幂等键时永远命中不了，参与时间判断时永远是「很久以前」。
///
/// 所以入口处统一过一遍这个函数：**只接受毫秒量级**的合理值，其余一律换成当前
/// 时间并告警。退化成「收到这条数据的时间」是安全的——它至少是真实的、单调的，
/// 不会让下游的时间窗口失效。
///
/// ## 参数
///
/// * `value` - 外部给的原始时间戳（毫秒语义），可能缺失。
///
/// ## 返回值
///
/// 一个保证落在 `[`[`MIN_PLAUSIBLE_MS`]`, 当前时间 + `[`MAX_FUTURE_SKEW_MS`]`]`
/// 之内的毫秒时间戳。
///
/// ## 示例
///
/// ```
/// use wheel_rs::time_utils::{normalize_ms, now_ms};
///
/// // 合理值原样返回
/// let given = now_ms() - 60_000;
/// assert_eq!(normalize_ms(Some(given)), given);
///
/// // 缺失、0、秒级都换成当前时间
/// assert!(normalize_ms(None) >= now_ms() - 1000);
/// assert!(normalize_ms(Some(0)) >= now_ms() - 1000);
/// assert!(normalize_ms(Some(1_800_000_000)) >= now_ms() - 1000);
/// ```
pub fn normalize_ms(value: Option<u64>) -> u64 {
    let now = now_ms();
    let Some(value) = value else {
        return now;
    };

    if is_plausible_ms(value) {
        return value;
    }

    warn!(
        "时间戳 {value} 不是合理的毫秒时间戳（不在 {MIN_PLAUSIBLE_MS}..={} 内），改用当前时间 {now}",
        now.saturating_add(MAX_FUTURE_SKEW_MS)
    );
    now
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 合理的毫秒时间戳原样保留：对端给的才是「什么时候发生的」
    #[test]
    fn keeps_plausible_millis() {
        // 一分钟前：既有毫秒量级，又不依赖运行时的具体时间
        let given = now_ms() - 60_000;
        assert_eq!(normalize_ms(Some(given)), given);
    }

    /// 缺失、0、秒级、明显超前都要换成当前时间
    #[test]
    fn replaces_implausible_values_with_now() {
        let now = now_ms();

        assert!(normalize_ms(None) >= now);
        assert!(normalize_ms(Some(0)) >= now);
        // 秒级时间戳：10 位，量级不对
        assert!(normalize_ms(Some(1_800_000_000)) >= now);
        // 明显超前的未来时间
        assert!(normalize_ms(Some(now + MAX_FUTURE_SKEW_MS * 100)) >= now);
    }

    /// 边界：2020-01-01 本身合法，前一毫秒就不可信
    #[test]
    fn min_boundary() {
        assert!(is_plausible_ms(MIN_PLAUSIBLE_MS));
        assert!(!is_plausible_ms(MIN_PLAUSIBLE_MS - 1));
        assert_eq!(normalize_ms(Some(MIN_PLAUSIBLE_MS)), MIN_PLAUSIBLE_MS);
    }

    /// 纳秒当毫秒用是最常见的量级错误，必须挡住
    #[test]
    fn rejects_nanoseconds() {
        assert!(!is_plausible_ms(now_ms() * 1_000_000));
    }
}
