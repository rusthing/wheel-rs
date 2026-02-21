use thiserror::Error;

#[derive(Error, Debug)]
pub enum StrError {
    #[error("String cannot be empty")]
    Empty,
    /// 无效的格式
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

/// 驼峰格式
#[derive(PartialEq)]
pub enum CamelFormat {
    /// 大驼峰
    Upper,
    /// 小驼峰
    Lower,
}

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
                    // 处理缩写情况：检查下一个字符
                    if i + 1 < chars.len() {
                        let next_ch = chars[i + 1];
                        // 如果下一个字符是小写，说明当前大写字母是新单词开始
                        if next_ch.is_ascii_lowercase() && current_word.len() > 1 {
                            let last_char = current_word.pop().unwrap();
                            words.push(current_word.clone());
                            current_word.clear();
                            current_word.push(last_char);
                        }
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
