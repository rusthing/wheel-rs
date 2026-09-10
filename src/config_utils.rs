//! # 配置工具模块
//!
//! 提供 `config` 配置对象的差异比较与变化判断工具。

use config::{Config, Map, Value, ValueKind};

/// 比较两个 `Config` 值，使用库自带的 `Value` 树进行递归比较。
/// 返回一个 Map，键为以 "." 分隔的全路径 key（如 "a.b.c"），值为新的 Value。
/// 如果 Map 为空，表示两个配置完全相同。
/// 被移除的 key 对应的值用 `ValueKind::Nil` 表示。
pub fn diff_config(old: &Config, new: &Config) -> Map<String, Value> {
    // 获取旧配置的顶层 Table
    let old_table = match &old.cache.kind {
        ValueKind::Table(table) => table,
        _ => return Map::new(),
    };
    // 获取新配置的顶层 Table
    let new_table = match &new.cache.kind {
        ValueKind::Table(table) => table,
        _ => return Map::new(),
    };

    let mut changed = Map::new();
    // 从顶层开始递归比较，prefix 初始为空
    diff_table(old_table, new_table, "", &mut changed);
    changed
}

/// 递归比较两个 Table，将差异收集到 `changed` 中。
/// `prefix` 为当前层级的全路径前缀（如 "a.b"），首次调用时传空字符串。
fn diff_table(
    old_table: &Map<String, Value>,
    new_table: &Map<String, Value>,
    prefix: &str,
    changed: &mut Map<String, Value>,
) {
    // 遍历新表中的所有 key，检查新增和变更
    for (key, new_val) in new_table {
        // 构造当前 key 的全路径
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        match old_table.get(key) {
            Some(old_val) => {
                // 如果两个值都是 Table，递归比较子表
                if let (ValueKind::Table(old_sub), ValueKind::Table(new_sub)) =
                    (&old_val.kind, &new_val.kind)
                {
                    diff_table(old_sub, new_sub, &full_key, changed);
                } else if old_val != new_val {
                    // 值不同（非 Table），记录变更
                    changed.insert(full_key, new_val.clone());
                }
                // 值相同，不记录
            }
            None => {
                // 新表中存在但旧表中不存在的 key，记录为新增
                changed.insert(full_key, new_val.clone());
            }
        }
    }

    // 遍历旧表，找出被移除的 key
    for key in old_table.keys() {
        // 构造当前 key 的全路径
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        // 旧表中存在但新表中不存在的 key，记录为移除（用 Nil 表示）
        if !new_table.contains_key(key) && !changed.contains_key(&full_key) {
            changed.insert(full_key, Value::new(None, ValueKind::Nil));
        }
    }
}

/// # 判断配置是否发生变化
///
/// 检查 [`diff_config`] 返回的差异 Map 中是否存在以给定前缀开头的 key。
///
/// ## 参数
///
/// * `key_prefix` - 要匹配的全路径 key 前缀（如 "a.b"）
/// * `changed` - [`diff_config`] 返回的差异 Map
///
/// ## 返回值
///
/// 若存在以 `key_prefix` 开头的 key 则返回 `true`，否则返回 `false`。
pub fn has_config_changed(key_prefix: &str, changed: &Map<String, Value>) -> bool {
    changed.keys().any(|key| key.starts_with(key_prefix))
}
