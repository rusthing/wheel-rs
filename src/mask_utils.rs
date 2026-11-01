//! # 脱敏工具模块
//!
//! 手机号、身份证号、邮箱、姓名、银行卡号、IP 的展示用脱敏，以及密码/盐/令牌类的
//! 整串遮蔽。只负责「把一段明文字符串变成可展示的遮蔽字符串」，不涉及权限判定——
//! 谁能看到明文是业务层的事（见文末）。
//!
//! ## 约定
//!
//! - **按字符处理，不按字节**：中文姓名按 `char` 截取，不会切出半个 UTF-8 字节
//! - **遮蔽长度不保留原文长度**：手机号固定 `***`… 这类思路会让「几位数」变成信息，
//!   这里的做法是逐字符替换成等长的 `*`，长度本身不敏感、且不用记每个字段的规则
//! - **保留位不足时整体遮住**：字符串短于「保留前 N + 保留后 M」时，宁可全遮也不漏字符
//! - 脱敏结果**不可逆**，只能展示：不要回写数据库，也不要拿来当查询条件
//!
//! ## 在框架里的位置
//!
//! 脱敏是**出口**问题，不是存储问题。库里的明文/密文是业务的源数据，一旦把脱敏值写回去，
//! 就等于把假数据变成了真数据。所以调用点只有两类：
//!
//! 1. 序列化时：Vo 上的敏感字段用包装类型承载（`Serialize` 输出本模块的结果，
//!    `Display`/`Debug` 也输出脱敏串，只有显式的 `reveal()` 才拿原文）
//! 2. 写日志前：`op_log`/`login_log` 这类留存久、查看权限宽的记录，**入库前**就脱敏；
//!    `tracing` 打印 DTO 时要小心 `Debug` 会绕过 `Serialize`，敏感字段的 `Debug` 也得脱敏
//!
//! ## 示例
//!
//! ```
//! use wheel_rs::mask_utils::{mask_email, mask_id_card, mask_mobile, mask_secret};
//!
//! assert_eq!(mask_mobile("13812345678"), "138****5678");
//! assert_eq!(mask_id_card("11010519491231002X"), "110105********002X");
//! assert_eq!(mask_email("zhang.san@example.com"), "z***n@example.com");
//! assert_eq!(mask_secret("a1b2c3d4e5f6"), "********");
//! ```

use std::net::Ipv4Addr;
use std::str::FromStr;

/// # 用 `*` 遮蔽字符串的中间部分
///
/// 保留前 `keep_front` 个字符与后 `keep_back` 个字符，中间逐字符替换为 `*`。
///
/// ## 参数
///
/// * `s` - 待脱敏的原文
/// * `keep_front` - 保留的前缀字符数
/// * `keep_back` - 保留的后缀字符数
///
/// ## 返回值
///
/// 脱敏后的新字符串。原文为空时返回空串；当 `keep_front + keep_back` 不小于字符数时，
/// **整串都遮蔽**——保留位不够就说明这段值太短，露出任何一位都可能直接暴露原文。
///
/// ## 示例
///
/// ```
/// use wheel_rs::mask_utils::mask_with;
///
/// assert_eq!(mask_with("abcdefgh", 2, 2, '*'), "ab****gh");
/// assert_eq!(mask_with("张三", 1, 0, '*'), "张*");
/// assert_eq!(mask_with("ab", 1, 1, '*'), "**"); // 保留位占满，全遮
/// ```
pub fn mask_with(s: &str, keep_front: usize, keep_back: usize, mask_char: char) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    if n == 0 {
        return String::new();
    }
    if keep_front + keep_back >= n {
        return std::iter::repeat_n(mask_char, n).collect();
    }
    let mut out = String::with_capacity(s.len());
    out.extend(&chars[..keep_front]);
    out.extend(std::iter::repeat_n(mask_char, n - keep_front - keep_back));
    out.extend(&chars[n - keep_back..]);
    out
}

/// # 用 `*` 遮蔽字符串的中间部分
///
/// [`mask_with`] 的 `*` 版本，最常用的一层。
///
/// ## 示例
///
/// ```
/// use wheel_rs::mask_utils::mask;
///
/// assert_eq!(mask("abcdefgh", 3, 4), "abc*efgh"); // 8 位里只遮掉中间的 1 位
/// ```
pub fn mask(s: &str, keep_front: usize, keep_back: usize) -> String {
    mask_with(s, keep_front, keep_back, '*')
}

/// # 手机号脱敏
///
/// 保留前 3 位（号段）与后 4 位，中间 4 位遮蔽：`13812345678` → `138****5678`。
/// 非 11 位的输入不报错，按同样的保留位规则处理（短号码会被整体遮住）。
///
/// ## 示例
///
/// ```
/// use wheel_rs::mask_utils::mask_mobile;
///
/// assert_eq!(mask_mobile("13812345678"), "138****5678");
/// assert_eq!(mask_mobile(""), "");
/// ```
pub fn mask_mobile(s: &str) -> String {
    mask(s, 3, 4)
}

/// # 身份证号脱敏
///
/// 保留前 6 位（地址码）与后 4 位（顺序码 + 校验位）：
/// `11010519491231002X` → `110105********002X`。
///
/// 之所以保留地址码：业务上常要按地区做筛选与核对；出生日期段与顺序码必须遮住，
/// 那两段才是能定位到个人的部分。
///
/// ## 示例
///
/// ```
/// use wheel_rs::mask_utils::mask_id_card;
///
/// assert_eq!(mask_id_card("11010519491231002X"), "110105********002X");
/// ```
pub fn mask_id_card(s: &str) -> String {
    mask(s, 6, 4)
}

/// # 银行卡号脱敏
///
/// 只保留后 4 位：`4111111111111111` → `************1111`。
/// 卡号前 6 位是发卡行标识（BIN），对业务有用，但它同时缩小了可枚举范围，
/// 面向用户的展示一律只留后 4 位。
///
/// ## 示例
///
/// ```
/// use wheel_rs::mask_utils::mask_bank_card;
///
/// assert_eq!(mask_bank_card("4111111111111111"), "************1111");
/// ```
pub fn mask_bank_card(s: &str) -> String {
    mask(s, 0, 4)
}

/// # 邮箱脱敏
///
/// 本地部分保留首尾各 1 个字符、中间固定为 `***`（不按原长度补星，避免暴露用户名长度），
/// 域名原样保留：`zhang.san@example.com` → `z***n@example.com`。
///
/// 域名不遮：它通常是公开信息，遮了反而让用户认不出是哪个邮箱。
///
/// ## 示例
///
/// ```
/// use wheel_rs::mask_utils::mask_email;
///
/// assert_eq!(mask_email("zhang.san@example.com"), "z***n@example.com");
/// assert_eq!(mask_email("ab@example.com"), "a***@example.com");
/// assert_eq!(mask_email("a@example.com"), "***@example.com");
/// assert_eq!(mask_email("not-an-email"), "************"); // 无 @ 时整体遮住
/// ```
pub fn mask_email(s: &str) -> String {
    let Some((local, domain)) = s.rsplit_once('@') else {
        return mask(s, 0, 0);
    };
    let chars: Vec<char> = local.chars().collect();
    let masked_local = match chars.len() {
        0 => "***".to_string(),
        1 => "***".to_string(),
        2 => format!("{}***", chars[0]),
        n => format!("{}***{}", chars[0], chars[n - 1]),
    };
    format!("{masked_local}@{domain}")
}

/// # 姓名脱敏
///
/// 只保留姓氏（首字），其余每字一个 `*`：`张三` → `张*`，`阿依古丽·买买提` → `阿*******`。
///
/// 复姓（`欧阳`、`司马`）会被遮掉第二个字，这是刻意的：复姓表得随人口普查更新，
/// 按表匹配既容易漏又会暴露更多字符，不如统一只留首字。
///
/// ## 示例
///
/// ```
/// use wheel_rs::mask_utils::mask_name;
///
/// assert_eq!(mask_name("张三"), "张*");
/// assert_eq!(mask_name("阿依古丽·买买提"), "阿*******");
/// assert_eq!(mask_name("张"), "*"); // 单字姓名整体遮住
/// ```
pub fn mask_name(s: &str) -> String {
    mask(s, 1, 0)
}

/// # 密码 / 盐 / 令牌类脱敏
///
/// **整串遮蔽，不保留任何字符，也不保留原长度**：一律输出固定的 8 个 `*`，
/// 空串输出空串。
///
/// 这与前面的「等长遮蔽」刻意不同：手机号漏出 3 位还剩 10⁴ 种可能，而密码散列值
/// 只要露出一个字符就可能缩短暴力破解的搜索空间；同理，`login_pswd_salt` 这类
/// 定长盐值一旦暴露长度，就等于告诉对方用的是哪套散列参数。
///
/// ## 用途
///
/// 这类值**本来就不该进任何 Vo**——接口不回密码是设计而不是脱敏。本函数覆盖的是
/// 剩下那几个容易漏的口子：
///
/// - DTO / Model 的 `Debug`（`tracing` 打印请求体时 `Debug` 会绕过 `Serialize`）
/// - 审计记录的请求快照（`op_log` 里存了改密请求的 body）
/// - 出错时把输入回显给调用方（校验失败提示里带上了原值）
///
/// ## 示例
///
/// ```
/// use wheel_rs::mask_utils::mask_secret;
///
/// assert_eq!(mask_secret("a1b2c3d4e5f6"), "********");
/// assert_eq!(mask_secret("x"), "********"); // 长度不外露
/// assert_eq!(mask_secret(""), ""); // 空值保持为空，表示「没设置」
/// ```
pub fn mask_secret(s: &str) -> String {
    if s.is_empty() {
        String::new()
    } else {
        SECRET_MASK.to_string()
    }
}

/// [`mask_secret`] 的输出：固定 8 个 `*`，与原文长度无关。
pub const SECRET_MASK: &str = "********";

/// # IPv4 地址脱敏
///
/// 保留前两段：`192.168.1.10` → `192.168.*.*`。内网排查时前两段足够定位网段，
/// 后两段能唯一指向主机，必须遮住。
///
/// 输入不是合法 IPv4（含 IPv6）时整体遮蔽，不做「按点切分」的近似处理。
///
/// ## 示例
///
/// ```
/// use wheel_rs::mask_utils::mask_ip;
///
/// assert_eq!(mask_ip("192.168.1.10"), "192.168.*.*");
/// assert_eq!(mask_ip("2001:db8::1"), "***********"); // IPv6 整体遮住
/// ```
pub fn mask_ip(s: &str) -> String {
    match Ipv4Addr::from_str(s) {
        Ok(ip) => {
            let o = ip.octets();
            format!("{}.{}.*.*", o[0], o[1])
        }
        Err(_) => mask(s, 0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_with_bounds() {
        assert_eq!(mask("abcdefgh", 0, 0), "********");
        assert_eq!(mask("abcdefgh", 8, 0), "********"); // 保留位占满，全遮
        assert_eq!(mask("abcdefgh", 3, 5), "********"); // 超出长度，全遮
        assert_eq!(mask("abc", 1, 1), "a*c");
        assert_eq!(mask("", 1, 1), "");
    }

    #[test]
    fn mask_with_is_char_based() {
        // 按 char 而不是字节切，中文不会被切坏
        assert_eq!(mask("张三丰", 1, 1), "张*丰");
        assert_eq!(mask("阿依古丽·买买提", 1, 1), "阿******提");
    }

    #[test]
    fn mask_custom_char() {
        assert_eq!(mask_with("13812345678", 3, 4, '#'), "138####5678");
    }

    #[test]
    fn mobile_and_friends() {
        assert_eq!(mask_mobile("13812345678"), "138****5678");
        assert_eq!(mask_id_card("11010519491231002X"), "110105********002X");
        assert_eq!(mask_bank_card("4111111111111111"), "************1111");
        assert_eq!(mask_name("张三"), "张*");
    }

    #[test]
    fn email_edge_cases() {
        assert_eq!(mask_email("zhang.san@example.com"), "z***n@example.com");
        assert_eq!(mask_email("ab@example.com"), "a***@example.com");
        assert_eq!(mask_email("@example.com"), "***@example.com");
        assert_eq!(mask_email("no-at-sign"), "**********");
    }

    #[test]
    fn ip_edge_cases() {
        assert_eq!(mask_ip("10.0.0.1"), "10.0.*.*");
        assert_eq!(mask_ip("2001:db8::1"), "***********");
        assert_eq!(mask_ip("not-an-ip"), "*********");
    }

    #[test]
    fn secret_hides_length() {
        // 与等长遮蔽不同：输出长度固定，原文长度不外露
        assert_eq!(mask_secret("a1b2c3d4e5f6"), SECRET_MASK);
        assert_eq!(mask_secret("x"), SECRET_MASK);
        assert_eq!(mask_secret("a").len(), mask_secret(&"b".repeat(64)).len());
        assert_eq!(mask_secret(""), "");
    }
}
