use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// # 将 `Option<Option<T>>` 序列化
///
/// - `None` → 序列化为 null（配合 `skip_serializing_if` 可实现跳过字段）
/// - `Some(None)` → 序列化为 null（字段存在但值为空）
/// - `Some(Some(v))` → 序列化 v 的值
///
/// 建议在 struct 字段上配合使用：
/// `#[serde(default, skip_serializing_if = "Option::is_none")]`
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
/// - key 不存在 → `None`（走 `Default::default()`）
/// - key 存在值为 null → `Some(None)`
/// - key 存在有值 → `Some(Some(v))`
pub fn deserialize<'de, T, D>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<T>::deserialize(d).map(Some)
}
