//! # 拼音工具模块
//!
//! 把中文姓名转成检索用的全拼与简拼：
//!
//! - 全拼：小写、音节间无分隔（张三 → `zhangsan`）
//! - 简拼：每个字取拼音首字母、大写无分隔（张三 → `ZS`）
//!
//! 姓名里的字母、数字原样保留（全拼转小写、简拼转大写），空白与标点忽略。
//!
//! 拼音库按单字取常用读音，无法据词判断多音字；作为姓氏时读音不同的常见字
//! 在本模块内做覆盖（单、曾、解、仇、查……），其余字按默认读音处理。
//!
//! ## 示例
//!
//! ```
//! use wheel_rs::pinyin_utils::name_pinyin;
//!
//! assert_eq!(
//!     name_pinyin("张三"),
//!     (Some("ZS".to_string()), Some("zhangsan".to_string()))
//! );
//! // 多音字姓氏按姓氏读音
//! assert_eq!(
//!     name_pinyin("单雄信"),
//!     (Some("SXX".to_string()), Some("shanxiongxin".to_string()))
//! );
//! ```

use pinyin::ToPinyin;

/// 常见多音字姓氏的读音覆盖（拼音库默认读音与姓氏读音不一致的字）
const SURNAME_PINYIN: &[(char, &str)] = &[
    ('单', "shan"),
    ('曾', "zeng"),
    ('解', "xie"),
    ('仇', "qiu"),
    ('查', "zha"),
    ('区', "ou"),
    ('朴', "piao"),
    ('乐', "yue"),
    ('覃', "qin"),
    ('缪', "miao"),
    ('盖', "ge"),
    ('尉', "yu"),
    ('折', "she"),
    ('秘', "bi"),
    ('句', "gou"),
    ('繁', "po"),
    ('隗', "wei"),
    ('员', "yun"),
    ('会', "kuai"),
    ('洗', "xian"),
    ('郇', "huan"),
    ('祭', "zhai"),
    ('重', "chong"),
    ('长', "zhang"),
    ('黑', "he"),
    ('贲', "ben"),
    ('藏', "zang"),
];

/// # 计算姓名的全拼与简拼
///
/// ## 参数
/// * `name` - 姓名原文，首尾空白会被忽略
///
/// ## 返回值
/// 返回 `(简拼, 全拼)`：
/// - 姓名空白（或只有标点、分隔符）时返回 `(None, None)`；
/// - 汉字取拼音，第一个字优先查多音字姓氏表；
/// - 字母、数字原样保留（全拼小写、简拼大写），空白与标点忽略。
///
/// 返回值不截断，按调用方列宽自行处理。
///
/// ## 示例
/// ```
/// use wheel_rs::pinyin_utils::name_pinyin;
///
/// assert_eq!(name_pinyin("  "), (None, None));
/// assert_eq!(
///     name_pinyin("李雷 A·B"),
///     (Some("LLAB".to_string()), Some("lileiab".to_string()))
/// );
/// ```
pub fn name_pinyin(name: &str) -> (Option<String>, Option<String>) {
    let name = name.trim();
    if name.is_empty() {
        return (None, None);
    }

    let mut full = String::with_capacity(name.len() * 4);
    let mut abbr = String::with_capacity(name.chars().count());
    for (index, ch) in name.chars().enumerate() {
        let pinyin = if index == 0 {
            surname_pinyin(ch).or_else(|| ch.to_pinyin().map(|pinyin| pinyin.plain()))
        } else {
            ch.to_pinyin().map(|pinyin| pinyin.plain())
        };
        match pinyin {
            Some(pinyin) => {
                full.push_str(pinyin);
                if let Some(first) = pinyin.chars().next() {
                    abbr.push(first.to_ascii_uppercase());
                }
            }
            None if ch.is_ascii_alphanumeric() => {
                full.push(ch.to_ascii_lowercase());
                abbr.push(ch.to_ascii_uppercase());
            }
            // 空白、间隔号、标点等直接忽略
            None => {}
        }
    }

    if full.is_empty() {
        return (None, None);
    }
    (Some(abbr), Some(full))
}

/// 查多音字姓氏表，非姓氏返回 `None`
fn surname_pinyin(ch: char) -> Option<&'static str> {
    SURNAME_PINYIN
        .iter()
        .find(|(surname, _)| *surname == ch)
        .map(|(_, pinyin)| *pinyin)
}

#[cfg(test)]
mod tests {
    use super::name_pinyin;

    #[test]
    fn converts_common_name() {
        assert_eq!(
            name_pinyin("张三"),
            (Some("ZS".to_string()), Some("zhangsan".to_string()))
        );
    }

    #[test]
    fn applies_polyphonic_surname() {
        assert_eq!(
            name_pinyin("单雄信"),
            (Some("SXX".to_string()), Some("shanxiongxin".to_string()))
        );
        assert_eq!(
            name_pinyin("曾国藩"),
            (Some("ZGF".to_string()), Some("zengguofan".to_string()))
        );
    }

    #[test]
    fn keeps_ascii_and_skips_separators() {
        assert_eq!(
            name_pinyin("李雷 A·B"),
            (Some("LLAB".to_string()), Some("lileiab".to_string()))
        );
    }

    #[test]
    fn blank_name_yields_none() {
        assert_eq!(name_pinyin("   "), (None, None));
        assert_eq!(name_pinyin("·"), (None, None));
    }
}
