//! # 字符串工具模块
//!
//! 提供字符串格式转换工具：驼峰命名拆分、`snake_case` 转 `PascalCase` 等。

use thiserror::Error;

/// # 字符串解析错误
///
/// 字符串格式转换过程中可能出现的错误。
#[derive(Error, Debug)]
pub enum StrError {
    /// 输入为空字符串
    #[error("String cannot be empty")]
    Empty,
    /// 无效的格式
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

/// # 驼峰格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CamelFormat {
    /// 大驼峰
    Upper,
    /// 小驼峰
    Lower,
}

/// # 将驼峰格式的字符串分割成单词数组
///
/// 按大写字母边界将驼峰字符串拆分为单词，支持处理连续大写字母（缩写）场景：
/// 连续大写段之后紧跟小写字母时，最后一个大写字母视为新单词的首字母。
/// 例如 `"parseXML"` 拆分为 `["parse", "XML"]`，`"XMLParser"` 拆分为 `["XML", "Parser"]`，
/// `"helloWorld"` 拆分为 `["hello", "World"]`。
///
/// ## 参数
///
/// * `s` - 待拆分的驼峰格式字符串
/// * `format` - 驼峰格式类型（[`CamelFormat::Upper`] 大驼峰 / [`CamelFormat::Lower`] 小驼峰）
///
/// ## 返回值
///
/// 拆分成功返回单词数组；单词由连续的字母或数字组成。
///
/// ## 错误
///
/// * 输入为空字符串时返回 [`StrError::Empty`]
/// * 首字符与 `format` 要求的大小写不符、或包含非字母数字字符时返回 [`StrError::InvalidFormat`]
///
/// ## 示例
///
/// ```
/// use wheel_rs::str_utils::{split_camel_case, CamelFormat};
///
/// let words = split_camel_case("helloWorld", CamelFormat::Lower).unwrap();
/// assert_eq!(words, vec!["hello", "World"]);
///
/// // 连续大写（缩写）作为一个整体，后面的单词正常切分
/// let words = split_camel_case("XMLParser", CamelFormat::Upper).unwrap();
/// assert_eq!(words, vec!["XML", "Parser"]);
/// ```
pub fn split_camel_case(s: &str, format: CamelFormat) -> Result<Vec<String>, StrError> {
    if s.is_empty() {
        return Err(StrError::Empty);
    }

    let chars: Vec<char> = s.chars().collect();

    if CamelFormat::Upper == format && !chars[0].is_ascii_uppercase() {
        // 首字符必须是大写字母
        return Err(StrError::InvalidFormat(
            "First character must be uppercase".to_string(),
        ));
    } else if CamelFormat::Lower == format && !chars[0].is_ascii_lowercase() {
        // 首字符必须是小写字母
        return Err(StrError::InvalidFormat(
            "First character must be lowercase".to_string(),
        ));
    }

    let mut words = Vec::new();
    let mut current_word = String::new();

    for (i, ch) in chars.iter().enumerate() {
        // 只允许字母和数字
        if !ch.is_ascii_alphanumeric() {
            return Err(StrError::InvalidFormat(format!(
                "Invalid character '{ch}' at position {i}"
            )));
        }

        if ch.is_ascii_uppercase() {
            // 新单词开始的条件：
            // 1. 不是第一个字符
            // 2. 前一个字符是小写字母或数字
            // 3. 或者是连续大写字母的结束（处理缩写）
            if i > 0 {
                let prev_ch = chars[i - 1];
                if prev_ch.is_ascii_lowercase() || prev_ch.is_ascii_digit() {
                    // 普通单词分隔
                    if !current_word.is_empty() {
                        words.push(current_word.clone());
                        current_word.clear();
                    }
                } else if prev_ch.is_ascii_uppercase() {
                    // 处理缩写情况：连续大写之后紧跟小写，说明当前大写字母是
                    // 新单词的首字母，应当在此处断开。
                    // 例如 "XMLParser" 在 'P' 处断开为 "XML" + "Parser"。
                    let next_is_lower = chars
                        .get(i + 1)
                        .is_some_and(|next| next.is_ascii_lowercase());
                    if next_is_lower && current_word.len() > 1 {
                        words.push(std::mem::take(&mut current_word));
                    }
                }
            }
        }
        current_word.push(*ch);
    }

    // 添加最后一个单词
    if !current_word.is_empty() {
        words.push(current_word);
    }

    Ok(words)
}

/// # `snake_case` → `PascalCase`：`deleted_at` → `DeletedAt`
///
/// 按下划线切分，丢弃空段（因此 `"a__b"`、`"_a"` 都能正确处理），
/// 每段首字符转大写后拼接。
pub fn snake_to_pascal(s: &str) -> String {
    s.split('_')
        .filter(|seg| !seg.is_empty())
        .map(|seg| {
            let mut c = seg.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upper(s: &str) -> Vec<String> {
        split_camel_case(s, CamelFormat::Upper).unwrap()
    }

    fn lower(s: &str) -> Vec<String> {
        split_camel_case(s, CamelFormat::Lower).unwrap()
    }

    #[test]
    fn splits_plain_camel() {
        assert_eq!(lower("helloWorld"), vec!["hello", "World"]);
        assert_eq!(upper("HelloWorld"), vec!["Hello", "World"]);
        assert_eq!(lower("abc123Def"), vec!["abc123", "Def"]);
    }

    /// 缩写必须整体保留，不能被拆成「前一个字母 + 后半段」：
    /// 曾经 `"XMLParser"` 会被切成 `["XM", "LParser"]`。
    #[test]
    fn keeps_abbreviation_whole() {
        assert_eq!(upper("XMLParser"), vec!["XML", "Parser"]);
        assert_eq!(upper("HTTPServer"), vec!["HTTP", "Server"]);
        assert_eq!(lower("myURLParser"), vec!["my", "URL", "Parser"]);
        assert_eq!(lower("parseXML"), vec!["parse", "XML"]);
    }

    #[test]
    fn single_word() {
        assert_eq!(upper("Xml"), vec!["Xml"]);
        assert_eq!(upper("XML"), vec!["XML"]);
    }

    #[test]
    fn rejects_bad_input() {
        assert!(matches!(
            split_camel_case("", CamelFormat::Upper),
            Err(StrError::Empty)
        ));
        assert!(matches!(
            split_camel_case("helloWorld", CamelFormat::Upper),
            Err(StrError::InvalidFormat(_))
        ));
        assert!(matches!(
            split_camel_case("Hello_World", CamelFormat::Upper),
            Err(StrError::InvalidFormat(_))
        ));
    }

    #[test]
    fn snake_to_pascal_works() {
        assert_eq!(snake_to_pascal("deleted_at"), "DeletedAt");
        assert_eq!(snake_to_pascal("a__b"), "AB");
        assert_eq!(snake_to_pascal(""), "");
    }
}
