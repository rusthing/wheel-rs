//! # 自定义序列化/反序列化器，用于处理 `Option<Option<T>>` 类型的数据
//!
//! 此模块提供 `Option<Option<T>>` 类型的序列化和反序列化实现，用于区分
//! 「字段不存在」（外层 `None`）与「字段存在但值为空」（`Some(None)`）两种情况。
//!
//! ## 为什么三个函数必须配套使用
//!
//! `#[crud_dto]` 会把 `Option<T>` 字段包装成 `Option<Option<T>>`，靠**外层**
//! `Option` 表达「DTO 里到底有没有这个字段」——这是部分更新（modify）语义的基础：
//! 没传的字段不更新，显式传 `null` 的字段才置空。
//!
//! 早期只生成了 `deserialize_with`，没生成 `serialize_with` / `skip_serializing_if`，
//! 于是客户端未设置的字段被 serde 默认序列化成 JSON `null`，服务端再用 `deserialize`
//! 还原成 `Some(None)`——**「字段没传」和「字段传了 null」在传输中被抹平成同一个值**，
//! 所有未设置的 `Option<String>` 字段都会被 UPDATE 成 NULL。
//! 例如 worker 上报任务执行结果时不设 `executor_instance`，
//! INSERT 时写进去的执行器实例号就被紧随其后的 UPDATE 抹成 NULL。
//!
//! 修复方式就是三个函数配套：
//!
//! ```text
//! #[serde(
//!     deserialize_with    = "option_option_serde::deserialize",
//!     serialize_with      = "option_option_serde::serialize",
//!     skip_serializing_if = "option_option_serde::is_none"
//! )]
//! ```
//!
//! - 外层 `None`（未设置）→ 被 [`is_none`] 跳过，JSON 里**没有这个 key**
//!   → 服务端走 `#[serde(default)]` 得到 `None` → `ActiveValue::NotSet`，列不更新；
//! - `Some(None)`（显式置空）→ 序列化成 `null` → 服务端得 `Some(None)`
//!   → `ActiveValue::Set(None)`，列被置空；
//! - `Some(Some(v))` → 序列化成 `v` → 服务端得 `Some(Some(v))`，列被设为 `v`。
//!
//! 注意：只有 `Option<String>` 字段会中招。`Option<u64>` / `Option<Duration>` 等类型
//! 没有被自动挂 `deserialize_with`，走 serde 默认行为（`null` → 外层 `None`），
//! 本来就是对的。

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// # 判断 `Option<Option<T>>` 的外层是否为 `None`
///
/// 配合 `#[serde(skip_serializing_if)]` 使用。只有「字段未设置」时才跳过序列化；
/// 「显式置空」（`Some(None)`）仍然会被发出（值为 `null`），
/// 服务端才能区分「没传」与「传了 null」。
pub fn is_none<T>(value: &Option<Option<T>>) -> bool {
    value.is_none()
}

/// # 将 `Option<Option<T>>` 序列化
///
/// - `None` → 序列化为 null（通常已被 [`is_none`] 跳过，走不到这里）
/// - `Some(None)` → 序列化为 null（字段存在但值为空）
/// - `Some(Some(v))` → 序列化 v 的值
pub fn serialize<S, T>(value: &Option<Option<T>>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    match value {
        Some(Some(v)) => v.serialize(serializer),
        Some(None) | None => serializer.serialize_none(),
    }
}

/// # 将 `Option<Option<T>>` 反序列化
///
/// - key 不存在 → `None`（依赖结构体上的 `#[serde(default)]`）
/// - key 存在值为 null → `Some(None)`
/// - key 存在有值 → `Some(Some(v))`
pub fn deserialize<'de, T, D>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<T>::deserialize(d).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serde::option_option_serde;
    use serde::{Deserialize, Serialize};

    /// 模拟 `#[crud_dto]` 生成的 ModifyDto 字段形态：`Option<String>` → `Option<Option<String>>`
    #[derive(Debug, Default, Serialize, Deserialize)]
    #[serde(default)]
    struct ModifyDto {
        #[serde(
            deserialize_with = "option_option_serde::deserialize",
            serialize_with = "option_option_serde::serialize",
            skip_serializing_if = "option_option_serde::is_none"
        )]
        name: Option<Option<String>>,
    }

    /// 未设置的字段必须整个不出现在 JSON 里，而不是变成 `"name":null`
    #[test]
    fn none_is_skipped_not_sent_as_null() {
        let dto = ModifyDto::default();
        let json = serde_json::to_string(&dto).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn some_none_is_sent_as_null() {
        let dto = ModifyDto { name: Some(None) };
        assert_eq!(serde_json::to_string(&dto).unwrap(), r#"{"name":null}"#);
    }

    #[test]
    fn some_some_is_sent_as_value() {
        let dto = ModifyDto {
            name: Some(Some("v".to_string())),
        };
        assert_eq!(serde_json::to_string(&dto).unwrap(), r#"{"name":"v"}"#);
    }

    /// 三种状态各走一圈，序列化 + 反序列化后必须原样还原
    #[test]
    fn round_trip_preserves_three_states() {
        let cases = [
            (ModifyDto::default(), None),
            (ModifyDto { name: Some(None) }, Some(None)),
            (
                ModifyDto {
                    name: Some(Some("v".to_string())),
                },
                Some(Some("v".to_string())),
            ),
        ];
        for (dto, expect) in cases {
            let json = serde_json::to_string(&dto).unwrap();
            let back: ModifyDto = serde_json::from_str(&json).unwrap();
            assert_eq!(back.name, expect, "json = {json}");
        }
    }

    /// 服务端收到不带该 key 的 body 时必须得到外层 None（= 不更新）
    #[test]
    fn absent_key_deserializes_to_none() {
        let back: ModifyDto = serde_json::from_str("{}").unwrap();
        assert_eq!(back.name, None);
    }

    #[test]
    fn is_none_only_true_for_outer_none() {
        assert!(is_none::<String>(&None));
        assert!(!is_none::<String>(&Some(None)));
        assert!(!is_none::<String>(&Some(Some(String::new()))));
    }
}
