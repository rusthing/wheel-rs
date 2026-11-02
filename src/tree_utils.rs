//! # 树编码（路径枚举）
//!
//! 用一列字符串表达整棵树的层级：每段固定 3 位数字，父节点的编码是子节点编码的
//! 前缀，`001` → `001002` → `001002003`。由此得到两个免费的性质：
//!
//! - **字典序就是树的前序**：`ORDER BY tree_code` 直接得到「父节点紧跟其整棵子树」，
//!   所以有树编码的表不需要额外的排序列。
//! - **子树查询是前缀匹配**：`LIKE '001002%'` 取到的正好是该节点及其整棵子树。
//!
//! 代价是「移动节点 = 改编码」，且会级联整棵子树。这个代价换来的就是上面两条，
//! 认定了路径枚举才用它；需要频繁移动的树应该用邻接表（`parent_id`）。
//!
//! ## 与列宽的关系
//!
//! 列宽 30 = 10 层 × 3 位，每层最多 999 个兄弟。这里的常量必须与数据库列宽一致，
//! 否则会出现「校验通过但写库截断」。改列宽时四处同改：模型、建表脚本、
//! [`regex_utils::pattern::TREE_CODE`] 与本模块。
//!
//! ## 本模块的边界
//!
//! 只做**字符串运算**，不碰数据库：给父编码与「当前最大的子编码」算出下一个子编码。
//! 「查库拿到最大子编码」是框架的事（`robotech` 的树编码工具），放在本模块里会
//! 把纯计算和一个具体的数据源绑死。

use crate::regex_utils::is_tree_code;

/// 每层的字符数（固定 3 位，不足左补 0）
pub const LEVEL_LEN: usize = 3;
/// 最大层数，与列宽 30 对齐
pub const MAX_LEVEL: usize = 10;
/// 最大总长度 = [`MAX_LEVEL`] × [`LEVEL_LEN`]
pub const MAX_LEN: usize = MAX_LEVEL * LEVEL_LEN;
/// 每层最大的段值（3 位数字的上限）
pub const MAX_SEQ: u32 = 999;
/// 自动生成时默认的段值步长
///
/// 紧贴：同一层按 `001`、`002`、`003` 顺序递增，一层最多 999 个兄弟。
/// 之后要在中间插节点，得取一个未占用的空位（如 `001` 与 `002` 之间没有，
/// 就要重排后面的兄弟——改编码会级联整棵子树），见模型列注释
/// 「中间插入按 010、020、030 递增留空位」的手工做法。
/// 导入已经排好序的历史数据时同样传 `1`，逐个紧贴。
pub const DEFAULT_STEP: u32 = 1;

/// # 判断是不是合法的树编码
///
/// 规则见 [`regex_utils::is_tree_code`]：1~10 组 3 位数字，总长 3 的倍数且不超过 30。
///
/// ## 参数
/// * `code` - 待判定的字符串
///
/// ## 返回值
/// * `bool` - 合法返回 `true`
pub fn is_valid(code: &str) -> bool {
    is_tree_code(code)
}

/// # 取层数
///
/// ## 参数
/// * `code` - 树编码
///
/// ## 返回值
/// * `usize` - 层数；非法编码返回 `0`
pub fn level(code: &str) -> usize {
    if is_valid(code) {
        code.len() / LEVEL_LEN
    } else {
        0
    }
}

/// # 取父编码
///
/// ## 参数
/// * `code` - 树编码
///
/// ## 返回值
/// * `Option<&str>` - 父节点的编码；根节点（只有一层）与非法编码返回 `None`
pub fn parent_code(code: &str) -> Option<&str> {
    if level(code) <= 1 {
        None
    } else {
        Some(&code[..code.len() - LEVEL_LEN])
    }
}

/// # 取根编码
///
/// ## 参数
/// * `code` - 树编码
///
/// ## 返回值
/// * `Option<&str>` - 最顶层那一段；非法编码返回 `None`
pub fn root_code(code: &str) -> Option<&str> {
    if is_valid(code) {
        Some(&code[..LEVEL_LEN])
    } else {
        None
    }
}

/// # 取最后一段的数值
///
/// ## 参数
/// * `code` - 树编码
///
/// ## 返回值
/// * `Option<u32>` - 末段数值（`001002` → `2`）；非法编码返回 `None`
pub fn last_seq(code: &str) -> Option<u32> {
    if !is_valid(code) {
        return None;
    }
    code[code.len() - LEVEL_LEN..].parse().ok()
}

/// # 从根到父的全部祖先编码
///
/// 不含自身，顺序从根开始：`001002003` → `["001", "001002"]`。
///
/// ## 参数
/// * `code` - 树编码
///
/// ## 返回值
/// * `Vec<&str>` - 祖先编码；根节点与非法编码返回空
pub fn ancestors(code: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut current = code;
    while let Some(parent) = parent_code(current) {
        out.push(parent);
        current = parent;
    }
    out.reverse();
    out
}

/// # 拼子编码
///
/// ## 参数
/// * `parent` - 父编码；传空串表示顶层节点
/// * `seq` - 该节点在父下的段值，`1..=999`
///
/// ## 返回值
/// * `Option<String>` - 子编码；`seq` 越界或超出最大层数时返回 `None`
pub fn child_code(parent: &str, seq: u32) -> Option<String> {
    if !(1..=MAX_SEQ).contains(&seq) {
        return None;
    }
    if parent.is_empty() {
        return Some(format!("{seq:0LEVEL_LEN$}"));
    }
    if !is_valid(parent) {
        return None;
    }
    if parent.len() + LEVEL_LEN > MAX_LEN {
        return None;
    }
    Some(format!("{parent}{seq:0LEVEL_LEN$}"))
}

/// # 算下一个子编码
///
/// 「父编码 + 当前同层最大的子编码」→ 下一个可用编码。查库拿到最大子编码是调用方
/// 的事（见 `robotech` 的树编码工具），这里只做递推。
///
/// ## 参数
/// * `parent` - 父编码；传空串表示顶层节点
/// * `max_child` - 该父下已有的最大子编码；`None` 表示还没有子节点
/// * `step` - 段值步长，见 [`DEFAULT_STEP`]；传 `0` 直接返回 `None`
///
/// ## 返回值
/// * `Option<String>` - 下一个子编码；段值超过 999、层数超过 10、`max_child`
///   不是 `parent` 的直接子节点，或任一编码非法时返回 `None`
pub fn next_child_code(parent: &str, max_child: Option<&str>, step: u32) -> Option<String> {
    if step == 0 {
        return None;
    }
    let base = match max_child {
        None => 0,
        Some(child) => {
            if !is_child_of(child, parent) {
                return None;
            }
            last_seq(child)?
        }
    };
    child_code(parent, base.saturating_add(step))
}

/// # 判断是不是直接子节点
///
/// ## 参数
/// * `code` - 待判定的编码
/// * `parent` - 假定的父编码；传空串表示判定「是不是顶层节点」
///
/// ## 返回值
/// * `bool` - 正好比父编码多一层且以父编码为前缀时返回 `true`
pub fn is_child_of(code: &str, parent: &str) -> bool {
    is_valid(code)
        && (parent.is_empty() || is_valid(parent))
        && code.len() == parent.len() + LEVEL_LEN
        && code.starts_with(parent)
}

/// # 判断是不是后代（任意深度）
///
/// ## 参数
/// * `code` - 待判定的编码
/// * `ancestor` - 假定的祖先编码
///
/// ## 返回值
/// * `bool` - `code` 以 `ancestor` 为前缀且更长时返回 `true`；相等返回 `false`
///   （一个节点不是自己的后代）
pub fn is_descendant_of(code: &str, ancestor: &str) -> bool {
    is_valid(code)
        && is_valid(ancestor)
        && code.len() > ancestor.len()
        && code.starts_with(ancestor)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 层级与合法性
    #[test]
    fn level_and_valid() {
        assert_eq!(level("001"), 1);
        assert_eq!(level("001002003"), 3);
        assert_eq!(level(&"001".repeat(10)), 10);
        // 非法：长度不是 3 的倍数、超长、带分隔符
        assert_eq!(level("0010020034"), 0);
        assert_eq!(level("001.002"), 0);
        assert_eq!(level(&"001".repeat(11)), 0);
        assert_eq!(level(""), 0);
        assert!(!is_valid("0010020034"));
    }

    /// 父子关系：根没有父，其余取末段之前
    #[test]
    fn parent_and_root() {
        assert_eq!(parent_code("001"), None);
        assert_eq!(parent_code("001002"), Some("001"));
        assert_eq!(parent_code("001002003"), Some("001002"));
        assert_eq!(parent_code(""), None);
        assert_eq!(root_code("001002003"), Some("001"));
        assert_eq!(root_code("bad"), None);
    }

    /// 祖先链：从根到父，不含自身
    #[test]
    fn ancestor_chain() {
        assert_eq!(ancestors("001002003"), ["001", "001002"]);
        assert!(ancestors("001").is_empty());
        assert!(ancestors("bad").is_empty());
    }

    /// 末段数值
    #[test]
    fn seq_of_last_segment() {
        assert_eq!(last_seq("001002"), Some(2));
        assert_eq!(last_seq("010020"), Some(20));
        assert_eq!(last_seq("999"), Some(999));
        assert_eq!(last_seq("0010020034"), None);
    }

    /// 拼子编码：定长补零、顶层、越界
    #[test]
    fn build_child_code() {
        assert_eq!(child_code("", 10).as_deref(), Some("010"));
        assert_eq!(child_code("001", 2).as_deref(), Some("001002"));
        assert_eq!(child_code("001", 999).as_deref(), Some("001999"));
        // 段值越界
        assert_eq!(child_code("001", 0), None);
        assert_eq!(child_code("001", 1000), None);
        // 父非法
        assert_eq!(child_code("00a", 1), None);
        // 超出 10 层：父已经是 10 层，不能再往下
        let deepest = "001".repeat(10);
        assert_eq!(child_code(&deepest, 1), None);
        // 9 层父可以再挂一层
        let expected = "001".repeat(9) + "001";
        assert_eq!(child_code(&"001".repeat(9), 1).as_deref(), Some(expected.as_str()));
    }

    /// 递推：步长与留空位
    #[test]
    fn next_child_follows_step() {
        // 默认步长 1，紧贴：第一个是 001
        assert_eq!(next_child_code("", None, DEFAULT_STEP).as_deref(), Some("001"));
        assert_eq!(next_child_code("001", None, DEFAULT_STEP).as_deref(), Some("001001"));
        // 有子节点：在最大值上加步长
        assert_eq!(
            next_child_code("001", Some("001002"), DEFAULT_STEP).as_deref(),
            Some("001003")
        );
        // 手工留空位：步长 10
        assert_eq!(next_child_code("", None, 10).as_deref(), Some("010"));
        assert_eq!(next_child_code("001", Some("001010"), 10).as_deref(), Some("001020"));
    }

    /// 递推的失败路径：满了、传错、非法
    #[test]
    fn next_child_rejects() {
        // 步长 0 没有意义（会原地打转）
        assert_eq!(next_child_code("", None, 0), None);
        // 一层放满：999 之后再要就给不出
        assert_eq!(next_child_code("", Some("999"), DEFAULT_STEP), None);
        assert_eq!(next_child_code("", Some("999"), 1), None);
        // max_child 不是该父的直接子节点（隔代或别的分支）
        assert_eq!(next_child_code("001", Some("001002003"), DEFAULT_STEP), None);
        assert_eq!(next_child_code("001", Some("002001"), DEFAULT_STEP), None);
        // 编码非法
        assert_eq!(next_child_code("001", Some("0010023"), DEFAULT_STEP), None);
        assert_eq!(next_child_code("00a", None, DEFAULT_STEP), None);
    }

    /// 子节点与后代判定
    #[test]
    fn child_and_descendant() {
        assert!(is_child_of("001", ""));
        assert!(is_child_of("001002", "001"));
        assert!(!is_child_of("001002003", "001")); // 隔代不是直接子节点
        assert!(!is_child_of("001", "001"));

        assert!(is_descendant_of("001002003", "001"));
        assert!(is_descendant_of("001002", "001"));
        // 自己不是自己的后代
        assert!(!is_descendant_of("001", "001"));
        // 前缀相同但长度不对（不是整段）
        assert!(!is_descendant_of("0010020034", "001002"));
    }

    /// 定长保证字典序 = 前序：父紧跟其整棵子树
    #[test]
    fn lexical_order_is_dfs_order() {
        let mut codes = vec!["001002", "001", "001001", "002", "001002001"];
        codes.sort_unstable();
        assert_eq!(codes, ["001", "001001", "001002", "001002001", "002"]);
    }
}
