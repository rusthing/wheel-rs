//! # 前缀树（压缩前缀树 / Patricia Trie）模块
//!
//! 提供 [`PatriciaTrie`]：一种边标签可跨多个字符的压缩前缀树（radix tree），
//! 支持精确键查询、前缀遍历（自动补全）、重复键去重计数。
//!
//! 移植自 Java 版 `rebue.wheel.core.trie`，同时修正了原实现中的若干问题：
//! - `size()` 计数在重复插入 / 前缀节点转叶子时不再出错；
//! - `list()` 的前缀语义保持一致（返回所有以 `prefix` 开头的键的值）；
//! - 子节点改用有序 `Vec` + 按首字符二分定位，替代 `LinkedList` 线性扫描；
//! - 键按原样处理（不静默 `trim`），空键在 `get`/`list`/`put` 下有明确行为。

/// # 前缀树节点
///
/// 每个节点保存一段「边标签」（`key`，可为多个字符）、可选的叶子值以及有序子节点。
/// `value` 为 `Some` 表示该节点对应一个已插入的完整键（叶子）。
#[derive(Debug)]
struct Node<V> {
    /// 边标签：从父节点到本节点的剩余键片段，非空（根节点除外）
    key: String,
    /// 叶子值；`Some` 表示本节点是一个已插入的键
    value: Option<V>,
    /// 子节点，按首字符升序排列，便于二分定位
    children: Vec<Node<V>>,
}

impl<V> Node<V> {
    fn new(key: String, value: Option<V>) -> Self {
        Self {
            key,
            value,
            children: Vec::new(),
        }
    }
}

/// # 最长公共前缀长度（按字节计）
///
/// 逐字符比较 `a` 与 `b`，返回共同前缀所占字节数（必为 UTF-8 字符边界）。
fn lcp(a: &str, b: &str) -> usize {
    let mut n = 0usize;
    for (ca, cb) in a.chars().zip(b.chars()) {
        if ca != cb {
            break;
        }
        n += ca.len_utf8();
    }
    n
}

/// # 压缩前缀树
///
/// 键为任意非空字符串；`size()` 表示已插入的叶子（键）个数，重复键只算一次。
#[derive(Debug)]
pub struct PatriciaTrie<V> {
    root: Node<V>,
    size: usize,
}

impl<V> Default for PatriciaTrie<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> PatriciaTrie<V> {
    /// 创建一个空的前缀树
    pub fn new() -> Self {
        Self {
            root: Node::new(String::new(), None),
            size: 0,
        }
    }

    /// # 叶子（键）个数
    pub fn size(&self) -> usize {
        self.size
    }

    /// 是否没有键
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// # 插入键值对
    ///
    /// 若键已存在则覆盖其值并返回 `false`（`size()` 不变）；
    /// 若键为新键则返回 `true`（`size()` 加一）。空键不插入、返回 `false`。
    ///
    /// ## 示例
    /// ```
    /// use wheel_rs::trie::PatriciaTrie;
    ///
    /// let mut t = PatriciaTrie::new();
    /// assert!(t.put("apple", 1));
    /// assert!(!t.put("apple", 2));   // 覆盖，size 不变
    /// assert_eq!(t.get("apple"), Some(&2));
    /// assert_eq!(t.size(), 1);
    /// ```
    pub fn put(&mut self, key: &str, value: V) -> bool {
        if key.is_empty() {
            return false;
        }
        if insert(&mut self.root, key, value) {
            self.size += 1;
            true
        } else {
            false
        }
    }

    /// # 精确查询
    ///
    /// 返回键对应的值的引用；键不存在或为空返回 `None`。
    pub fn get(&self, key: &str) -> Option<&V> {
        get(&self.root, key)
    }

    /// # 精确查询（可变引用）
    pub fn get_mut(&mut self, key: &str) -> Option<&mut V> {
        get_mut(&mut self.root, key)
    }

    /// 是否包含某个键
    pub fn contains(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// # 前缀查询
    ///
    /// 返回所有键以 `prefix` 为前缀的值集合（自动补全）。
    /// 空前缀返回树中全部值。注意：与 Java 版不同，这里会继续收集匹配节点之下的所有后代。
    ///
    /// ## 示例
    /// ```
    /// use wheel_rs::trie::PatriciaTrie;
    ///
    /// let mut t = PatriciaTrie::new();
    /// t.put("ab", 1);
    /// t.put("abc", 2);
    /// t.put("abd", 3);
    /// t.put("x", 4);
    /// let mut got = t.list("ab");
    /// got.sort();
    /// assert_eq!(got, vec![&1, &2, &3]);
    /// ```
    pub fn list(&self, prefix: &str) -> Vec<&V> {
        let mut out = Vec::new();
        collect_prefix(&self.root, prefix, &mut out);
        out
    }

    /// # 返回全部已插入的键
    pub fn keys(&self) -> Vec<String> {
        let mut out = Vec::new();
        collect_keys(&self.root, &mut String::new(), &mut out);
        out
    }
}

/// 向 `node` 插入 `key`；返回 `true` 表示新增了一个叶子（`size` 应 +1）。
fn insert<V>(node: &mut Node<V>, key: &str, value: V) -> bool {
    let Some(first) = key.chars().next() else {
        return false;
    };
    // 子节点按首字符升序，二分定位：pos 为「首字符 >= first」的首个下标
    let pos = node
        .children
        .partition_point(|c| c.key.chars().next().unwrap() < first);
    let matched = node
        .children
        .get(pos)
        .is_some_and(|c| c.key.chars().next() == Some(first));

    if !matched {
        // 无同首字符兄弟：直接在有序位置插入新叶子
        node.children
            .insert(pos, Node::new(key.to_string(), Some(value)));
        return true;
    }

    let ck = node.children[pos].key.len();
    let kl = key.len();
    let l = lcp(&node.children[pos].key, key);

    if l == ck {
        if l == kl {
            // 键与子节点边标签完全相同：覆盖或首次置为叶子
            let child = &mut node.children[pos];
            let added = child.value.is_none();
            child.value = Some(value);
            return added;
        }
        // 键继续深入当前子节点（ck < kl）
        return insert(&mut node.children[pos], &key[ck..], value);
    }

    // l < ck：键在子节点边标签内部发生分歧，需要按公共前缀分裂
    let tail = node.children[pos].key[l..].to_string();
    node.children[pos].key.truncate(l); // l 为字符边界，安全
    let child = &mut node.children[pos];

    // 原子节点整体下移为「后缀节点」，继承其叶子值与子节点
    let inner = Node {
        key: tail,
        value: child.value.take(),
        children: std::mem::take(&mut child.children),
    };

    if kl == l {
        // 键恰好是该公共前缀：分裂点节点本身成为叶子
        child.value = Some(value);
        child.children.push(inner);
        return true;
    }

    // 键比公共前缀更长：分裂点下挂两个分支（保留有序性）
    let leaf = Node::new(key[l..].to_string(), Some(value));
    let tail_first = inner.key.chars().next().unwrap();
    let leaf_first = key[l..].chars().next().unwrap();
    if tail_first < leaf_first {
        child.children.push(inner);
        child.children.push(leaf);
    } else {
        child.children.push(leaf);
        child.children.push(inner);
    }
    true
}

/// 精确查询（不可变）
fn get<'a, V>(node: &'a Node<V>, key: &str) -> Option<&'a V> {
    let first = key.chars().next()?;
    let pos = node
        .children
        .partition_point(|c| c.key.chars().next().unwrap() < first);
    let child = node.children.get(pos)?;
    if child.key.chars().next() != Some(first) {
        return None;
    }
    let l = lcp(&child.key, key);
    let ck = child.key.len();
    if l != ck {
        return None; // 键未完整覆盖边标签（键更短或已分歧）
    }
    if key.len() == ck {
        child.value.as_ref()
    } else {
        get(child, &key[ck..])
    }
}

/// 精确查询（可变）
fn get_mut<'a, V>(node: &'a mut Node<V>, key: &str) -> Option<&'a mut V> {
    let first = key.chars().next()?;
    let pos = node
        .children
        .partition_point(|c| c.key.chars().next().unwrap() < first);
    let child = node.children.get_mut(pos)?;
    if child.key.chars().next() != Some(first) {
        return None;
    }
    let l = lcp(&child.key, key);
    let ck = child.key.len();
    if l != ck {
        return None;
    }
    if key.len() == ck {
        child.value.as_mut()
    } else {
        get_mut(child, &key[ck..])
    }
}

/// 收集所有以 `prefix` 为前缀的叶子值到 `out`。
fn collect_prefix<'a, V>(node: &'a Node<V>, prefix: &str, out: &mut Vec<&'a V>) {
    let Some(first) = prefix.chars().next() else {
        // 空前缀：整个子树都匹配
        collect_all(node, out);
        return;
    };
    let pos = node
        .children
        .partition_point(|c| c.key.chars().next().unwrap() < first);
    let Some(child) = node.children.get(pos) else {
        return;
    };
    if child.key.chars().next() != Some(first) {
        return;
    }

    let l = lcp(&child.key, prefix);
    let ck = child.key.len();
    let pl = prefix.len();

    if l == ck {
        // 前缀覆盖了整个边标签
        if pl == ck {
            // 前缀 == 边标签：该子树全部匹配
            collect_all(child, out);
        } else {
            // 前缀比边标签长：继续向下精确走一段
            collect_prefix(child, &prefix[ck..], out);
        }
    } else if l == pl {
        // 前缀是边标签的真前缀：该子树全部匹配
        collect_all(child, out);
    }
    // 其余情况：键与边标签在公共前缀后分歧，无匹配
}

/// 收集整个子树的所有叶子值到 `out`。
fn collect_all<'a, V>(node: &'a Node<V>, out: &mut Vec<&'a V>) {
    if let Some(v) = &node.value {
        out.push(v);
    }
    for c in &node.children {
        collect_all(c, out);
    }
}

/// 收集整个子树的所有键到 `out`（返回持有的 `String`，避免借用临时缓冲区）。
///
/// 完整键 = 从根到叶所有边标签的拼接，因此叶子处记录的是累计的整个 `prefix`。
fn collect_keys<V>(node: &Node<V>, prefix: &mut String, out: &mut Vec<String>) {
    let start = prefix.len();
    prefix.push_str(&node.key);
    if node.value.is_some() {
        out.push(prefix.clone());
    }
    for c in &node.children {
        collect_keys(c, prefix, out);
    }
    prefix.truncate(start);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_basic() {
        let mut t = PatriciaTrie::new();
        assert!(t.put("apple", "a"));
        assert!(t.put("apply", "b"));
        assert!(t.put("application", "c"));
        assert_eq!(t.size(), 3);
        assert_eq!(t.get("apple"), Some(&"a"));
        assert_eq!(t.get("apply"), Some(&"b"));
        assert_eq!(t.get("application"), Some(&"c"));
        assert_eq!(t.get("app"), None);
        assert_eq!(t.get("apples"), None);
    }

    #[test]
    fn duplicate_key_size_is_stable() {
        let mut t = PatriciaTrie::new();
        t.put("abc", 1);
        t.put("abc", 2);
        t.put("abc", 3);
        assert_eq!(t.size(), 1);
        assert_eq!(t.get("abc"), Some(&3));
    }

    #[test]
    fn prefix_becomes_leaf_size_correct() {
        // 修复 Java 版「内部前缀节点转叶子时 size 少计」的 bug
        let mut t = PatriciaTrie::new();
        t.put("abcd", 1);
        t.put("abxy", 2);
        assert_eq!(t.size(), 2);
        t.put("ab", 3);
        assert_eq!(t.size(), 3); // 应为 3，而非 Java 版误算的 2
        assert_eq!(t.get("ab"), Some(&3));
        assert_eq!(t.get("abcd"), Some(&1));
        assert_eq!(t.get("abxy"), Some(&2));
    }

    #[test]
    fn leaf_then_longer_child() {
        let mut t = PatriciaTrie::new();
        t.put("ab", 1);
        t.put("abc", 2);
        t.put("abd", 3);
        assert_eq!(t.size(), 3);
        assert_eq!(t.get("ab"), Some(&1));
        assert_eq!(t.get("abc"), Some(&2));
        assert_eq!(t.get("abd"), Some(&3));
    }

    #[test]
    fn list_prefix_collects_descendants() {
        // 修复 Java 版 list 在命中节点即返回、不收集后代的 bug
        let mut t = PatriciaTrie::new();
        t.put("ab", 1);
        t.put("abc", 2);
        t.put("abd", 3);
        t.put("x", 4);
        t.put("xyz", 5);
        t.put("yy", 6);

        let mut ab = t.list("ab");
        ab.sort();
        assert_eq!(ab, vec![&1, &2, &3]);

        let mut empty = t.list("");
        empty.sort();
        assert_eq!(empty, vec![&1, &2, &3, &4, &5, &6]);

        assert_eq!(t.list("xyz"), vec![&5]);
        assert_eq!(t.list("z"), Vec::<&i32>::new());
    }

    #[test]
    fn list_prefix_proper_prefix_of_edge() {
        // 前缀是边标签的真前缀时应收集整棵子树
        let mut t = PatriciaTrie::new();
        t.put("abcd", 1);
        t.put("abxy", 2);
        let mut a = t.list("ab");
        a.sort();
        assert_eq!(a, vec![&1, &2]);
        // "abc" 是 "abcd" 的前缀，但不是 "abxy" 的前缀（c != x）
        assert_eq!(t.list("abc"), vec![&1]);
        assert_eq!(t.list("abd"), Vec::<&i32>::new());
    }

    #[test]
    fn unicode_keys() {
        let mut t = PatriciaTrie::new();
        t.put("中文", 1);
        t.put("中文前缀", 2);
        t.put("其他", 3);
        t.put("😀", 4); // 增补平面字符（4 字节）
        assert_eq!(t.size(), 4);
        assert_eq!(t.get("中文"), Some(&1));
        assert_eq!(t.get("中文前缀"), Some(&2));
        assert_eq!(t.get("其他"), Some(&3));
        assert_eq!(t.get("😀"), Some(&4));
        assert_eq!(t.get("😁"), None);

        let mut zh = t.list("中文");
        zh.sort();
        assert_eq!(zh, vec![&1, &2]);

        let mut keys = t.keys();
        keys.sort();
        assert_eq!(keys, vec!["中文", "中文前缀", "其他", "😀"]);
    }

    #[test]
    fn empty_key_behavior() {
        let mut t = PatriciaTrie::new();
        assert!(!t.put("", 1));
        assert_eq!(t.get(""), None);
        assert_eq!(t.list("").len(), 0);
        assert!(t.is_empty());
    }

    #[test]
    fn get_mut_updates_in_place() {
        let mut t = PatriciaTrie::new();
        t.put("k", 1);
        *t.get_mut("k").unwrap() = 99;
        assert_eq!(t.get("k"), Some(&99));
        assert_eq!(t.size(), 1);
        assert!(t.contains("k"));
    }

    #[test]
    fn prefix_pressure() {
        // 大量共享前缀，验证分裂与查找稳定性
        let mut t = PatriciaTrie::new();
        let base = "a".repeat(100);
        for i in 0..200 {
            t.put(&format!("{base}-{i}"), i);
        }
        assert_eq!(t.size(), 200);
        for i in 0..200 {
            assert_eq!(t.get(&format!("{base}-{i}")), Some(&i));
        }
        // 前缀 "{base}-1" 匹配 i=1、10..=19、100..=199，共 1+10+100=111 个
        assert_eq!(t.list(&format!("{base}-1")).len(), 111);
    }
}
