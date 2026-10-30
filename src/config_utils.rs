//! # 配置工具模块
//!
//! 提供 `config` 配置对象的差异比较与变化判断工具。

use config::{Config, Map, Value, ValueKind};

/// # 比较两个 `Config` 值的差异
///
/// 使用库自带的 `Value` 树进行递归比较。
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
        if !new_table.contains_key(key) {
            changed.insert(full_key, Value::new(None, ValueKind::Nil));
        }
    }
}

/// # 判断配置是否发生变化
///
/// 检查 [`diff_config`] 返回的差异 Map 中是否存在位于给定前缀之下的 key。
///
/// 前缀按 **`.` 分段边界**匹配：`key_prefix` 为 `"a.b"` 时，`"a.b"` 与 `"a.b.c"`
/// 都算命中，而 `"a.bc"` **不算**——否则 `"a"` 会误命中 `"abc"`，导致无关配置的
/// 变更被当成自己这一块的变更。
///
/// ## 参数
///
/// * `key_prefix` - 要匹配的全路径 key 前缀（如 "a.b"）；末尾的 `.` 会被忽略
/// * `changed` - [`diff_config`] 返回的差异 Map
///
/// ## 返回值
///
/// 若存在位于 `key_prefix` 之下的 key 则返回 `true`；空前缀表示「任意变更」。
pub fn has_config_changed(key_prefix: &str, changed: &Map<String, Value>) -> bool {
    let prefix = key_prefix.trim_end_matches('.');
    if prefix.is_empty() {
        return !changed.is_empty();
    }
    changed.keys().any(|key| {
        key == prefix
            || key
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('.'))
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    fn config_of(pairs: &[(&str, &str)]) -> Config {
        let mut table = Map::new();
        for (k, v) in pairs {
            table.insert(
                (*k).to_string(),
                Value::new(None, ValueKind::String((*v).to_string())),
            );
        }
        let mut cfg = Config::default();
        cfg.cache.kind = ValueKind::Table(table);
        cfg
    }

    #[test]
    fn diff_reports_added_changed_removed() {
        let old = config_of(&[("same", "1"), ("changed", "old"), ("removed", "x")]);
        let new = config_of(&[("same", "1"), ("changed", "new"), ("added", "y")]);
        let changed = diff_config(&old, &new);

        assert!(!changed.contains_key("same"));
        assert_eq!(changed["changed"].to_string(), "new");
        assert_eq!(changed["added"].to_string(), "y");
        assert!(matches!(changed["removed"].kind, ValueKind::Nil));
    }

    #[test]
    fn prefix_matches_on_key_boundary() {
        let changed = diff_config(&config_of(&[]), &config_of(&[("abc", "1")]));
        // "a" 不是 "abc" 的父级前缀
        assert!(!has_config_changed("a", &changed));
        // 完全同名、以及以 "." 分隔的子级才算命中
        assert!(has_config_changed("abc", &changed));

        let nested = diff_config(&config_of(&[]), &config_of(&[("a.b.c", "1")]));
        assert!(has_config_changed("a", &nested));
        assert!(has_config_changed("a.b", &nested));
        assert!(has_config_changed("a.b.", &nested));
        assert!(!has_config_changed("a.bc", &nested));
    }

    #[test]
    fn empty_prefix_means_any_change() {
        assert!(!has_config_changed(
            "",
            &diff_config(&config_of(&[]), &config_of(&[]))
        ));
        assert!(has_config_changed(
            "",
            &diff_config(&config_of(&[]), &config_of(&[("a", "1")]))
        ));
    }
}
