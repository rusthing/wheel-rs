//! # 正则校验模块
//!
//! 常用字段格式的**唯一定义**：邮箱、手机号、身份证号、统一社会信用代码、银行卡号、
//! URL、树编码、账号与编码等。框架给 DTO 参数挂校验、业务层判空判格式时都从这里取，
//! 避免各处各写一份正则，导致「接口放行、数据库拒绝」或前后端口径不一致。
//!
//! ## 三层结构
//!
//! - [`pattern`]：`&'static str` 正则源串。校验框架需要的是一段正则文本（如 OpenAPI 的
//!   `pattern` 字段、DTO 上的正则校验属性），直接从常量取，不要自己再拼
//! - 模块顶层的 [`EMAIL`]、[`MOBILE_CN`] 等 `pub static`：同一模式的编译结果
//!   （[`LazyLock`]），业务代码直接 `is_match`，不要每次 `Regex::new`
//! - `is_*` 函数：**带语义**的校验入口。除结构正则外还做正则表达不了的判断——
//!   身份证的校验位与出生日期、统一社会信用代码的校验位、银行卡的 Luhn 校验、
//!   邮箱的总长上限、密码的字符类别——因此比单独跑一遍正则更严格
//!
//! ## 口径约定表
//!
//! | 校验 | 规则 | 正则能单独判定 |
//! |------|------|----------------|
//! | [`is_email`] | RFC 5322 简化版；总长 ≤254、本地部分 ≤64、域名标签各 ≤63 | 否，长度另判 |
//! | [`is_mobile_cn`] | 中国大陆手机号：`1` + `3..=9` + 9 位数字，共 11 位 | 是 |
//! | [`is_landline_cn`] | 中国大陆固话：3 位区号配 8 位号码、4 位区号配 7 位号码，可选分机 | 是 |
//! | [`is_id_card_cn`] | 18 位身份证：结构 + 出生日期真实存在且不在未来 + ISO 7064 校验位 | 否 |
//! | [`is_uscc`] | 18 位统一社会信用代码：字符集 + GB 32100 模 31 校验位 | 否 |
//! | [`is_bank_card`] | 银行卡号 16~19 位数字 + Luhn 校验 | 否 |
//! | [`is_plate_cn`] | 机动车号牌：省份简称 + 发牌机关字母 + 序号（7 位普通 / 8 位新能源 / 带后缀），只认大写 | 否，序号需含数字 |
//! | [`is_url_http`] | `http://` 或 `https://` 开头的绝对 URL，不含空白 | 是 |
//! | [`is_tree_code`] | 树编码：每层固定 3 位数字，1~10 层，总长 3/6/…/30 | 是 |
//! | [`is_uuid`] | 8-4-4-4-12 十六进制，忽略大小写 | 是 |
//! | [`is_account_name`] | 登录名：字母开头，字母/数字/下划线，4~32 位 | 是 |
//! | [`is_code`] | 通用编码：字母/数字/`_`/`.`/`-`，1~50 位 | 是 |
//! | [`is_chinese_name`] | 中文姓名：汉字与间隔号 `·`，2~30 字 | 是 |
//! | [`is_password`] | 8~64 位可打印 ASCII（不含空格），且同时含字母与数字 | 否，字符类别另判 |
//! | [`is_ip`] / [`is_ipv4`] / [`is_ipv6`] | 用 [`std::net`] 解析，比正则更严（拒绝前导 0） | 否 |
//!
//! ## 为什么有些规则「正则能判」还要再写函数
//!
//! 同一个模式在框架里可能以两种形式出现：DTO 校验拿到的是字符串正则，业务层拿到的是
//! 编译好的 [`Regex`]。两者必须完全等价，所以 `is_mobile_cn` 这类函数只做
//! `MOBILE_CN.is_match(s)` 一件事——它存在的意义是**统一入口**，让调用点不必关心
//! 用的是哪个静态实例，也不必担心有人漏了 `^`/`$` 而让 `is_match` 变成子串匹配。
//!
//! ## 模型注释里的引用方式
//!
//! 逻辑模型（`.ldm`）的列注释用 `@` 标记声明约束，框架据此生成 DTO 校验。本模块负责
//! 其中两类，**函数名必须与这里的 `is_*` 完全一致**：
//!
//! | 标记 | 指向 | 用在 |
//! |------|------|------|
//! | `@check is_email` | 本模块的 `is_email`（经 `robotech::api::validators::email` 适配） | 谓词校验：带校验位、带长度判断、解析式判断的都在这里 |
//! | `@regex EMAIL` | [`pattern`] 里的 `EMAIL` | 只需要一段正则文本的场景（OpenAPI `pattern`、前端表单） |
//!
//! 二者的取舍：`@check` 认的是函数，比正则严（能带校验位），但要求校验框架能调 Rust
//! 函数；`@regex` 只给得出文本，跨语言也能用，但只能表达结构。模型里默认写 `@check`，
//! 需要给外部系统导出正则时才补 `@regex`。
//!
//! ## 示例
//!
//! ```
//! use wheel_rs::regex_utils::{is_email, is_id_card_cn, is_mobile_cn, is_tree_code};
//!
//! assert!(is_mobile_cn("13812345678"));
//! assert!(!is_mobile_cn("12812345678")); // 第二位不合法
//! assert!(is_email("zhang.san@example.com"));
//! assert!(is_id_card_cn("11010519491231002X")); // 含校验位
//! assert!(!is_id_card_cn("110105194912310021")); // 校验位错
//! assert!(is_tree_code("001002003")); // 3 层，每层 3 位
//! assert!(!is_tree_code("0010020034")); // 不是 3 的倍数
//! ```

use regex::Regex;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::sync::LazyLock;

use chrono::NaiveDate;

/// # 正则模式源串
///
/// 每个常量都是**整串匹配**（带 `^` 与 `$`）的正则文本，可以直接交给校验框架。
/// 需要「只判子串」的场景请自己去掉锚点，不要改这里的常量。
pub mod pattern {
    /// 邮箱地址（RFC 5322 简化版）。
    ///
    /// 本地部分：`A-Za-z0-9` 与 ``!#$%&'*+/=?^_`{|}~-``，点号只能出现在两个字符之间；
    /// 域名部分：至少两段，各段 1~63 字符、不以连字符开头结尾，顶级域为 2~63 个字母。
    /// **不含总长判断**（254），总长与本地部分长度上限由 [`super::is_email`] 另判。
    pub const EMAIL: &str = r"^[A-Za-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[A-Za-z0-9!#$%&'*+/=?^_`{|}~-]+)*@(?:[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?\.)+[A-Za-z]{2,63}$";

    /// 中国大陆手机号：`1` + `3..=9` + 9 位数字，共 11 位。
    ///
    /// 不含 `+86` 前缀与空格、连字符等分隔符——入库前请先归一化。
    pub const MOBILE_CN: &str = r"^1[3-9]\d{9}$";

    /// 中国大陆固定电话：区号 + 号码，可带 1~6 位分机。
    ///
    /// 区号与号码长度是**配对**的：3 位区号（`010`、`021` 等大城市）配 8 位号码，
    /// 4 位区号配 7 位号码。号码首位不能是 `0` 或 `1`。分隔符 `-` 可有可无。
    pub const LANDLINE_CN: &str = r"^0(?:\d{2}-?[2-9]\d{7}|\d{3}-?[2-9]\d{6})(?:-\d{1,6})?$";

    /// 18 位居民身份证号的**结构**（不含校验位验证）。
    ///
    /// 6 位地址码 + 8 位出生日期（年月日各段合法范围）+ 3 位顺序码 + 1 位校验位（`0-9` 或 `X`）。
    /// 校验位与日期真实性由 [`super::is_id_card_cn`] 另判。
    pub const ID_CARD_CN: &str =
        r"^[1-9]\d{5}(?:18|19|20)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01])\d{3}[\dXx]$";

    /// 18 位统一社会信用代码的**结构**（不含校验位验证）。
    ///
    /// 字符集为 `0-9` 与除 `I`、`O`、`S`、`V`、`Z` 之外的 21 个大写字母，
    /// 第 1~2 位登记管理部门 + 第 3~8 位登记管理机关行政区划码 + 第 9~17 位主体标识码 +
    /// 第 18 位校验码。校验位由 [`super::is_uscc`] 另判。
    pub const USCC: &str = r"^[0-9A-HJ-NPQRTUWXY]{2}\d{6}[0-9A-HJ-NPQRTUWXY]{10}$";

    /// 银行卡号**结构**：16~19 位纯数字。Luhn 校验由 [`super::is_bank_card`] 另判。
    pub const BANK_CARD: &str = r"^\d{16,19}$";

    /// 中国大陆机动车号牌（蓝牌/黄牌/新能源/挂车/教练/警用等常见形式）。
    ///
    /// 结构是「省份简称 + 发牌机关字母 + 序号」，序号有四种形式：
    ///
    /// | 形式 | 位数 | 例子 |
    /// |------|------|------|
    /// | 普通序号码（数字与字母混排） | 5 | `京A12345`、`粤B8F6Z2` |
    /// | 小型新能源（`D`/`F` 打头 + 5 位数字） | 6 | `京AD12345`、`沪AF67890` |
    /// | 大型新能源（5 位数字 + 末尾 `D`/`F`） | 6 | `京A12345D` |
    /// | 4 位数字 + 后缀汉字 | 5 | `京A1234挂`、`沪C8888学` |
    ///
    /// 发牌机关字母与序号字母都排除 `I`/`O`（与车管所口径一致，避免与 `1`/`0` 混淆）。
    /// 序号里的字母个数**不做限制**：各地对「字母不得超过 2 位」的执行尺度不一，
    /// 新能源牌也出现过带字母的号段，收紧要冒拒真的风险，而这里只能判结构、判不了真伪。
    ///
    /// **只认大写**。收小写会让同一辆车以 `京A12345` 与 `京a12345` 两条记录共存，
    /// 而库里的唯一约束（`dtw_vehicle.code`）是大小写敏感的，等于给重复留了口子；
    /// 归一化请在上游做（输入框转大写），校验层不做隐式转换。
    ///
    /// **不覆盖**：军牌（`V12345`、`BA12345` 等无省份简称的形式）、使馆牌（`使123456`）、
    /// 民航/港口等内部号牌。这些号段的规则不在民用号牌体系内，遇到时不要在业务侧放宽
    /// 本函数，改用「不做格式校验 + 人工核对」。
    pub const PLATE_CN: &str = r"^[京津沪渝冀豫云辽黑湘皖鲁新苏浙赣鄂桂甘晋蒙陕吉闽贵粤青藏川宁琼][A-HJ-NP-Z](?:[0-9A-HJ-NP-Z]{5}|[DF][0-9]{5}|[0-9]{5}[DF]|[0-9]{4}[挂学警领港澳试])$";

    /// `http://` 或 `https://` 开头的绝对 URL。
    ///
    /// 主机允许字母/数字/`.`/`-`（含 IPv4，不含带方括号的 IPv6 字面量），
    /// 可选端口与路径，整串不含空白与引号。
    pub const URL_HTTP: &str = r#"^https?://[A-Za-z0-9.\-]+(?::\d{1,5})?(?:/[^\s"'<>]*)?$"#;

    /// 树编码：每层固定 3 位数字，1~10 层，总长必须是 3 的倍数且不超过 30。
    ///
    /// `001` → `001002` → `001002003`。定长是「字典序即树的前序」的前提；
    /// 层级与字符数上限要和数据库列宽（`varchar(30)`）保持一致，改一处必须同步另一处。
    pub const TREE_CODE: &str = r"^(?:\d{3}){1,10}$";

    /// UUID：`8-4-4-4-12` 十六进制，大小写不敏感。
    pub const UUID: &str =
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$";

    /// 登录名：字母开头，只含字母/数字/下划线，4~32 位。
    ///
    /// 字母开头是为了避开「纯数字登录名与手机号、工号混淆」，下划线之外的符号一律不收，
    /// 避免大小写不敏感的系统里 `a-b` 与 `a_b` 被视为同一个名字。
    pub const ACCOUNT_NAME: &str = r"^[A-Za-z][A-Za-z0-9_]{3,31}$";

    /// 通用编码：字母/数字/下划线/点/连字符，1~50 位。
    ///
    /// 这类编码是给人看的业务编号（应用编码、组织编码、权限编码、工号等），
    /// 允许点与连字符；不像 [`ACCOUNT_NAME`] 那样限制首字符——`001` 这种纯数字工号是合法的。
    pub const CODE: &str = r"^[A-Za-z0-9_.\-]{1,50}$";

    /// 中文姓名：汉字与间隔号 `·`（少数民族姓名），2~30 字。
    ///
    /// 只覆盖 `U+4E00..=U+9FA5`（基本汉字区），生僻字（扩展 A/B 区）会被拒——
    /// 需要支持生僻字时不要在框架里放宽这条，改为「不做正则校验 + 人工审核」。
    pub const CHINESE_NAME: &str = r"^[\x{4e00}-\x{9fa5}·]{2,30}$";

    /// 可打印 ASCII 密码（8~64 位，不含空格）。
    ///
    /// **只判字符集与长度**。「至少含字母与数字」需要前瞻断言，而 Rust 的 `regex`
    /// 不支持前瞻，所以这条规则由 [`super::is_password`] 在代码里判定。
    pub const PASSWORD_CHARSET: &str = r"^[\x21-\x7e]{8,64}$";

    /// IPv4 点分十进制（各段 0~255，不允许前导 0）。
    ///
    /// 能用 [`super::is_ipv4`] 就别用这个正则：`std` 的解析更严（拒绝 `010.1.1.1` 这类
    /// 前导 0 与多余空白），且更快。这个常量只为「框架只吃正则」的场景保留。
    pub const IPV4: &str =
        r"^(?:(?:25[0-5]|2[0-4]\d|1\d{2}|[1-9]?\d)\.){3}(?:25[0-5]|2[0-4]\d|1\d{2}|[1-9]?\d)$";
}

/// 编译内置模式，失败只可能是常量写错，属于编码错误而非运行时错误。
macro_rules! compiled {
    ($($name:ident),+ $(,)?) => {
        $(
            #[doc = concat!("[`pattern::", stringify!($name), "`] 的编译结果。")]
            pub static $name: LazyLock<Regex> = LazyLock::new(|| {
                Regex::new(pattern::$name)
                    .expect(concat!("built-in regex is invalid: ", stringify!($name)))
            });
        )+
    };
}

compiled!(
    EMAIL,
    MOBILE_CN,
    LANDLINE_CN,
    ID_CARD_CN,
    USCC,
    BANK_CARD,
    PLATE_CN,
    URL_HTTP,
    TREE_CODE,
    UUID,
    ACCOUNT_NAME,
    CODE,
    CHINESE_NAME,
    PASSWORD_CHARSET,
    IPV4,
);

/// # 校验邮箱地址
///
/// 规则见 [`pattern::EMAIL`]，另加两条正则不便表达的上限：整串不超过 **254** 字节、
/// `@` 前的本地部分不超过 **64** 字节（RFC 5321 对投递路径的限制，超长的地址在实际
/// 投递中会被拒）。
///
/// ## 参数
///
/// * `s` - 待校验的地址，必须是「原样」地址：不允许前后空格、不允许 `张三 <a@b.com>`
///   这种带显示名的形式
///
/// ## 返回值
///
/// 合法返回 `true`，否则返回 `false`。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_email;
///
/// assert!(is_email("zhang.san+tag@example.co"));
/// assert!(!is_email("zhang..san@example.com")); // 连续点
/// assert!(!is_email("zhang@example")); // 没有顶级域
/// assert!(!is_email("  a@b.com")); // 带空格
/// ```
pub fn is_email(s: &str) -> bool {
    if s.len() > 254 {
        return false;
    }
    let Some((local, domain)) = s.rsplit_once('@') else {
        return false;
    };
    local.len() <= 64 && domain.len() <= 253 && EMAIL.is_match(s)
}

/// # 校验中国大陆手机号
///
/// 规则见 [`pattern::MOBILE_CN`]：`1` + `3..=9` + 9 位数字，共 11 位。
/// 不认 `+8613812345678`、`138-1234-5678`，入库前请先归一化。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_mobile_cn;
///
/// assert!(is_mobile_cn("13812345678"));
/// assert!(!is_mobile_cn("1381234567")); // 10 位
/// assert!(!is_mobile_cn("+8613812345678")); // 带国家码
/// ```
pub fn is_mobile_cn(s: &str) -> bool {
    MOBILE_CN.is_match(s)
}

/// # 校验中国大陆固定电话
///
/// 规则见 [`pattern::LANDLINE_CN`]：3 位区号配 8 位号码、4 位区号配 7 位号码，
/// 号码首位不能是 `0` 或 `1`（避免把 `010-1234567` 这种位数错位的号码放进来），
/// 可带 1~6 位分机。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_landline_cn;
///
/// assert!(is_landline_cn("010-88886666"));
/// assert!(is_landline_cn("05718888888"));
/// assert!(is_landline_cn("010-88886666-803"));
/// assert!(!is_landline_cn("010-8888666")); // 3 位区号配了 7 位号码
/// assert!(!is_landline_cn("10-88886666")); // 区号缺前导 0
/// ```
pub fn is_landline_cn(s: &str) -> bool {
    LANDLINE_CN.is_match(s)
}

/// # 校验 18 位居民身份证号
///
/// 三步都通过才算合法：
///
/// 1. 结构见 [`pattern::ID_CARD_CN`]（地址码首位非 0、出生日期各段在合法范围）
/// 2. 出生日期必须真实存在（`2 月 30 日`、`4 月 31 日` 会被拒），且不晚于今天
/// 3. 第 18 位校验位符合 ISO 7064:1983 MOD 11-2（加权因子 `7 9 10 5 8 4 2 1 6 3 7 9 10 5 8 4 2`，
///    余数映射 `10X98765432`），末位 `x` 按 `X` 处理
///
/// **不校验**地址码是否在现行行政区划表内：区划会撤并、新增，硬编码的表迟早过期，
/// 校验位本身已经能拦住绝大多数手误。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_id_card_cn;
///
/// assert!(is_id_card_cn("11010519491231002X"));
/// assert!(is_id_card_cn("44030819990101999x")); // 小写 x 可接受
/// assert!(!is_id_card_cn("110105194912310021")); // 校验位错
/// assert!(!is_id_card_cn("110105194902310021")); // 2 月 31 日
/// ```
pub fn is_id_card_cn(s: &str) -> bool {
    if !ID_CARD_CN.is_match(s) || s.len() != 18 {
        return false;
    }
    // 到这里 18 个字节都是 ASCII，可以按字节切片
    let digits = &s[..17];
    let (Ok(year), Ok(month), Ok(day)) = (
        digits[6..10].parse::<i32>(),
        digits[10..12].parse::<u32>(),
        digits[12..14].parse::<u32>(),
    ) else {
        return false;
    };
    let Some(birth) = NaiveDate::from_ymd_opt(year, month, day) else {
        return false;
    };
    if birth > chrono::Local::now().date_naive() {
        return false;
    }
    id_card_check_char(digits) == s.as_bytes()[17].to_ascii_uppercase()
}

/// ISO 7064:1983 MOD 11-2 的校验位，输入必须是 17 位数字。
fn id_card_check_char(digits: &str) -> u8 {
    const WEIGHTS: [u32; 17] = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
    const CHECK: &[u8; 11] = b"10X98765432";

    let sum: u32 = digits
        .bytes()
        .zip(WEIGHTS)
        .map(|(b, w)| u32::from(b - b'0') * w)
        .sum();
    CHECK[(sum % 11) as usize]
}

/// # 校验统一社会信用代码
///
/// 结构见 [`pattern::USCC`]，另加 GB 32100 规定的模 31 校验位：
/// 前 17 位按权重 `1 3 9 27 19 26 16 17 20 29 25 13 8 24 10 30 28` 加权求和，
/// 对 31 取模后用字符集 `0123456789ABCDEFGHJKLMNPQRTUWXY` 索引（余数为 0 时校验码为 `0`）。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_uscc;
///
/// assert!(is_uscc("91350100M000100Y43"));
/// assert!(!is_uscc("91350211M000100Y43")); // 校验位错
/// assert!(!is_uscc("91350100M000100Y4O")); // O 不在字符集内
/// ```
pub fn is_uscc(s: &str) -> bool {
    const CHARSET: &[u8; 31] = b"0123456789ABCDEFGHJKLMNPQRTUWXY";
    const WEIGHTS: [u32; 17] = [
        1, 3, 9, 27, 19, 26, 16, 17, 20, 29, 25, 13, 8, 24, 10, 30, 28,
    ];

    if !USCC.is_match(s) || s.len() != 18 {
        return false;
    }
    let mut sum = 0u32;
    for (b, w) in s.bytes().take(17).zip(WEIGHTS) {
        let Some(idx) = CHARSET.iter().position(|c| *c == b) else {
            return false;
        };
        sum += idx as u32 * w;
    }
    let rem = sum % 31;
    let check = if rem == 0 { 0 } else { 31 - rem };
    s.as_bytes()[17] == CHARSET[check as usize]
}

/// # 校验银行卡号
///
/// 16~19 位纯数字（见 [`pattern::BANK_CARD`]），且满足 Luhn（模 10）校验。
///
/// Luhn 能挡住绝大多数单个数字写错或相邻两位颠倒的情况，但**不能**证明卡号真实存在——
/// 真正的有效性只能靠发卡行或支付渠道的鉴权接口。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_bank_card;
///
/// assert!(is_bank_card("4111111111111111"));
/// assert!(!is_bank_card("4111111111111112")); // Luhn 不通过
/// assert!(!is_bank_card("411111111111111")); // 15 位
/// ```
pub fn is_bank_card(s: &str) -> bool {
    if !BANK_CARD.is_match(s) {
        return false;
    }
    // 从右往左，偶数位（0 起）加倍
    let mut sum = 0u32;
    for (i, b) in s.bytes().rev().enumerate() {
        let mut d = u32::from(b - b'0');
        if i % 2 == 1 {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
    }
    sum.is_multiple_of(10)
}

/// # 校验中国大陆机动车号牌
///
/// 结构见 [`pattern::PLATE_CN`]，另加一条正则不便表达（Rust 的 `regex` 没有前瞻）
/// 的判断：**序号里至少有一位数字**。5 位全字母的序号不在发放范围内，`京ABCDEF`
/// 这类乱填靠这一条拦下。
///
/// 只认合法形式，不能证明号牌真实存在——真正的有效性要靠行驶证或车管所核验。
/// 军牌、使馆牌、内部号牌不在覆盖范围，见 [`pattern::PLATE_CN`] 的说明。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_plate_cn;
///
/// assert!(is_plate_cn("京A12345")); // 普通
/// assert!(is_plate_cn("粤B8F6Z2")); // 数字与字母混排
/// assert!(is_plate_cn("京AD12345")); // 小型新能源
/// assert!(is_plate_cn("京A12345D")); // 大型新能源
/// assert!(is_plate_cn("沪C8888学")); // 教练车
/// assert!(!is_plate_cn("京I12345")); // 不用字母 I
/// assert!(!is_plate_cn("京a12345")); // 小写不算
/// assert!(!is_plate_cn("京ABCDEF")); // 序号全是字母
/// ```
pub fn is_plate_cn(s: &str) -> bool {
    if !PLATE_CN.is_match(s) {
        return false;
    }
    // 跳过省份简称与发牌机关字母，剩余部分（序号 + 可选后缀）必须含数字
    s.chars().skip(2).any(|c| c.is_ascii_digit())
}

/// # 校验 HTTP(S) URL
///
/// 规则见 [`pattern::URL_HTTP`]：必须带 `http://` 或 `https://`，主机为域名或 IPv4，
/// 允许端口与路径，整串不含空白、引号与尖括号。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_url_http;
///
/// assert!(is_url_http("https://example.com/a/b?c=1"));
/// assert!(is_url_http("http://192.168.1.10:8080/"));
/// assert!(!is_url_http("ftp://example.com"));
/// assert!(!is_url_http("/a/b")); // 相对路径不是绝对 URL
/// ```
pub fn is_url_http(s: &str) -> bool {
    URL_HTTP.is_match(s)
}

/// # 校验树编码
///
/// 每层固定 **3** 位数字、**不加分隔符**，按层顺序拼接；总长必须是 3 的倍数且不超过
/// **30**（即最多 10 层，每层最多 999 个兄弟）。
///
/// `001` → `001002` → `001002003`。定长保证了「字典序即树的前序」：父节点编码是子节点
/// 编码的前缀，`ORDER BY tree_code` 直接得到 DFS 序，`LIKE '001002%'` 取的正好是
/// 该节点及其整棵子树。
///
/// 这个长度上限必须和数据库列宽一致（`varchar(30)`）；改动时两处同改，否则会出现
/// 「校验通过但写库截断」或反过来。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_tree_code;
///
/// assert!(is_tree_code("001"));
/// assert!(is_tree_code("001002003"));
/// assert!(!is_tree_code("0010020034")); // 10 位，不是 3 的倍数
/// assert!(!is_tree_code("001.002")); // 带分隔符
/// assert!(!is_tree_code("0012003")); // 7 位
/// ```
pub fn is_tree_code(s: &str) -> bool {
    TREE_CODE.is_match(s)
}

/// # 校验 UUID 文本
///
/// 规则见 [`pattern::UUID`]，只认带连字符的标准形式（`8-4-4-4-12`），
/// 不认 `{}` 包裹、`urn:uuid:` 前缀与 32 位无连字符形式。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_uuid;
///
/// assert!(is_uuid("123e4567-e89b-12d3-a456-426614174000"));
/// assert!(!is_uuid("123e4567e89b12d3a456426614174000"));
/// ```
pub fn is_uuid(s: &str) -> bool {
    UUID.is_match(s)
}

/// # 校验登录名
///
/// 规则见 [`pattern::ACCOUNT_NAME`]：字母开头，字母/数字/下划线，4~32 位。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_account_name;
///
/// assert!(is_account_name("zhang_san"));
/// assert!(!is_account_name("1zhangsan")); // 数字开头
/// assert!(!is_account_name("zhang-san")); // 含连字符
/// assert!(!is_account_name("abc")); // 太短
/// ```
pub fn is_account_name(s: &str) -> bool {
    ACCOUNT_NAME.is_match(s)
}

/// # 校验通用业务编码
///
/// 规则见 [`pattern::CODE`]：字母/数字/下划线/点/连字符，1~50 位。
/// 与列宽 `varchar(50)` 对齐；用在别的长度上请先确认列宽。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_code;
///
/// assert!(is_code("SYS_ADMIN"));
/// assert!(is_code("001"));
/// assert!(is_code("a.b-c"));
/// assert!(!is_code("")); // 不能为空
/// assert!(!is_code("中文编码")); // 不含非 ASCII
/// ```
pub fn is_code(s: &str) -> bool {
    CODE.is_match(s)
}

/// # 校验中文姓名
///
/// 规则见 [`pattern::CHINESE_NAME`]：汉字与间隔号 `·`，2~30 字，不认空格与西文字母。
/// 少数民族姓名（`阿依古丽·买买提`）能过，带空格的外文名过不了——这类数据不要用本函数。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_chinese_name;
///
/// assert!(is_chinese_name("张三"));
/// assert!(is_chinese_name("阿依古丽·买买提"));
/// assert!(!is_chinese_name("张")); // 单字姓名不合法
/// assert!(!is_chinese_name("John Smith"));
/// ```
pub fn is_chinese_name(s: &str) -> bool {
    CHINESE_NAME.is_match(s)
}

/// # 校验密码（明文，入库前）
///
/// 两段规则：
///
/// 1. 字符集与长度见 [`pattern::PASSWORD_CHARSET`]：8~64 位可打印 ASCII，不含空格
/// 2. 至少各含一个字母与一个数字。**这条必须用代码判定**：Rust 的 `regex` 不支持前瞻
///    断言，写不出「至少含字母」的模式，用多个正则跑一遍反而更慢更难维护
///
/// 只用于「用户设置密码」这一步；校验通过后立刻用 SM3/SHA 系列摘要，摘要串不要再过本函数。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_password;
///
/// assert!(is_password("Abcd1234"));
/// assert!(is_password("abcd1234")); // 不强制大小写混用，有字母有数字即可
/// assert!(!is_password("abcdefgh")); // 没有数字
/// assert!(!is_password("12345678")); // 没有字母
/// assert!(!is_password("Abc 1234")); // 含空格
/// assert!(!is_password("Abc123")); // 太短
/// ```
pub fn is_password(s: &str) -> bool {
    if !PASSWORD_CHARSET.is_match(s) {
        return false;
    }
    let has_alpha = s.bytes().any(|b| b.is_ascii_alphabetic());
    let has_digit = s.bytes().any(|b| b.is_ascii_digit());
    has_alpha && has_digit
}

/// # 校验 IPv4 地址
///
/// 用 [`Ipv4Addr`] 解析，比正则严格：拒绝 `010.1.1.1` 这类前导 0、拒绝多余空白。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_ipv4;
///
/// assert!(is_ipv4("192.168.1.10"));
/// assert!(!is_ipv4("192.168.1.256"));
/// assert!(!is_ipv4("010.1.1.1"));
/// ```
pub fn is_ipv4(s: &str) -> bool {
    Ipv4Addr::from_str(s).is_ok()
}

/// # 校验 IPv6 地址
///
/// 用 [`Ipv6Addr`] 解析，支持压缩写法（`::`）与嵌入 IPv4 的形式。
/// IPv6 的正则几乎必然写错（压缩零、双冒号只能出现一次），这里不提供正则，只提供解析。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_ipv6;
///
/// assert!(is_ipv6("::1"));
/// assert!(is_ipv6("2001:db8::8a2e:370:7334"));
/// assert!(!is_ipv6("192.168.1.1"));
/// ```
pub fn is_ipv6(s: &str) -> bool {
    Ipv6Addr::from_str(s).is_ok()
}

/// # 校验 IPv4 或 IPv6 地址
///
/// 先按 IPv4 解析，失败再按 IPv6 解析；不接受带方括号的 `[::1]` 形式（那是 URL 里的写法）。
///
/// ## 示例
///
/// ```
/// use wheel_rs::regex_utils::is_ip;
///
/// assert!(is_ip("10.0.0.1"));
/// assert!(is_ip("::1"));
/// assert!(!is_ip("[::1]"));
/// ```
pub fn is_ip(s: &str) -> bool {
    is_ipv4(s) || is_ipv6(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_patterns_compile() {
        // 常量写错要到首次访问静态量时才 panic，这里逐个触发一遍，
        // 免得某个写错的正则等线上请求才暴露
        for re in [
            &*EMAIL,
            &*MOBILE_CN,
            &*LANDLINE_CN,
            &*ID_CARD_CN,
            &*USCC,
            &*BANK_CARD,
            &*PLATE_CN,
            &*URL_HTTP,
            &*TREE_CODE,
            &*UUID,
            &*ACCOUNT_NAME,
            &*CODE,
            &*CHINESE_NAME,
            &*PASSWORD_CHARSET,
            &*IPV4,
        ] {
            assert!(!re.as_str().is_empty());
        }
    }

    #[test]
    fn patterns_are_fully_anchored() {
        // 漏了锚点会退化成子串匹配，是最容易犯又最难发现的错
        for p in [
            pattern::EMAIL,
            pattern::MOBILE_CN,
            pattern::LANDLINE_CN,
            pattern::ID_CARD_CN,
            pattern::USCC,
            pattern::BANK_CARD,
            pattern::PLATE_CN,
            pattern::URL_HTTP,
            pattern::TREE_CODE,
            pattern::UUID,
            pattern::ACCOUNT_NAME,
            pattern::CODE,
            pattern::CHINESE_NAME,
            pattern::PASSWORD_CHARSET,
            pattern::IPV4,
        ] {
            assert!(p.starts_with('^') && p.ends_with('$'), "未锚定: {p}");
        }
    }

    #[test]
    fn email_rules() {
        assert!(is_email("a@b.co"));
        assert!(is_email("zhang.san+tag@sub.example.com"));
        assert!(!is_email("a@b")); // 无顶级域
        assert!(!is_email(".a@b.com")); // 本地部分以点开头
        assert!(!is_email("a..b@b.com")); // 连续点
        assert!(!is_email("a@-b.com")); // 域名标签以连字符开头
        assert!(!is_email(&format!("{}@b.com", "a".repeat(65)))); // 本地部分超 64
    }

    #[test]
    fn mobile_rules() {
        assert!(is_mobile_cn("13812345678"));
        assert!(is_mobile_cn("19912345678"));
        assert!(!is_mobile_cn("12812345678")); // 第二位 2
        assert!(!is_mobile_cn("138123456789")); // 12 位
        assert!(!is_mobile_cn("138 1234 5678"));
    }

    #[test]
    fn landline_rules() {
        assert!(is_landline_cn("010-88886666"));
        assert!(is_landline_cn("05718888888"));
        assert!(is_landline_cn("0755-8888888"));
        assert!(is_landline_cn("010-88886666-803"));
        assert!(!is_landline_cn("010-8888666")); // 3 位区号配了 7 位号码
        assert!(!is_landline_cn("0571-88888888")); // 4 位区号配了 8 位号码
        assert!(!is_landline_cn("10-88886666"));
    }

    #[test]
    fn id_card_rules() {
        assert!(is_id_card_cn("11010519491231002X"));
        assert!(is_id_card_cn("44030819990101999x")); // 小写 x
        assert!(!is_id_card_cn("110105194912310021")); // 校验位错
        assert!(!is_id_card_cn("110105194902310021")); // 2 月 31 日
        assert!(!is_id_card_cn("11010519491231002")); // 17 位
        assert!(!is_id_card_cn("1101051949123100XX")); // 顺序码里有字母
    }

    #[test]
    fn id_card_rejects_future_birth() {
        // 2099 年出生：结构与校验位都对，只有「日期在未来」这一条把它拦下
        assert!(!is_id_card_cn("110105209912310029"));
    }

    #[test]
    fn uscc_rules() {
        assert!(is_uscc("91350100M000100Y43"));
        assert!(!is_uscc("91350211M000100Y43")); // 校验位错
        assert!(!is_uscc("91350100M000100Y4O")); // O 不在字符集
        assert!(!is_uscc("91350100M000100Y4")); // 17 位
    }

    #[test]
    fn bank_card_rules() {
        assert!(is_bank_card("4111111111111111"));
        assert!(!is_bank_card("4111111111111112")); // Luhn 不过
        assert!(!is_bank_card("411111111111111")); // 15 位
        assert!(!is_bank_card("4111 1111 1111 1111")); // 带空格
    }

    #[test]
    fn plate_rules() {
        assert!(is_plate_cn("京A12345")); // 普通蓝牌/黄牌
        assert!(is_plate_cn("粤B8F6Z2")); // 数字与字母混排
        assert!(is_plate_cn("京AD12345")); // 小型新能源：D 打头
        assert!(is_plate_cn("沪AF67890")); // 小型新能源：F 打头
        assert!(is_plate_cn("京A12345D")); // 大型新能源：D 收尾
        assert!(is_plate_cn("京A1234挂")); // 挂车
        assert!(is_plate_cn("沪C8888学")); // 教练车
        assert!(is_plate_cn("粤Z1234港")); // 港澳入境
        assert!(!is_plate_cn("京I12345")); // 不用字母 I
        assert!(!is_plate_cn("京O12345")); // 不用字母 O
        assert!(!is_plate_cn("京a12345")); // 只认大写
        assert!(!is_plate_cn("京A1234")); // 只有 4 位数字
        assert!(!is_plate_cn("京A123456")); // 8 位但不是新能源形式
        assert!(!is_plate_cn("京ABCDEF")); // 序号全是字母
        assert!(!is_plate_cn("使123456")); // 使馆牌不在覆盖范围
        assert!(!is_plate_cn("WJ12345")); // 武警牌
        assert!(!is_plate_cn("京A 12345")); // 带空格
    }

    #[test]
    fn url_rules() {
        assert!(is_url_http("https://example.com"));
        assert!(is_url_http("http://192.168.1.10:8080/a/b?c=1#d"));
        assert!(!is_url_http("ftp://example.com"));
        assert!(!is_url_http("/a/b"));
        assert!(!is_url_http("https://example.com/a b")); // 含空格
    }

    #[test]
    fn tree_code_rules() {
        assert!(is_tree_code("001"));
        assert!(is_tree_code("001002003004005006"));
        // 10 层正好 30 字符，是上限
        assert!(is_tree_code(&"001".repeat(10)));
        assert!(!is_tree_code(&"001".repeat(11))); // 33 字符，超长
        assert!(!is_tree_code("0010020034"));
        assert!(!is_tree_code("001.002"));
    }

    #[test]
    fn tree_code_lexical_order_is_dfs_order() {
        // 定长的意义：字典序 = 前序，父节点紧跟其整棵子树
        let mut codes = vec!["001002", "001", "001001", "002", "001002001"];
        codes.sort_unstable();
        assert_eq!(codes, ["001", "001001", "001002", "001002001", "002"]);
    }

    #[test]
    fn uuid_rules() {
        assert!(is_uuid("123e4567-e89b-12d3-a456-426614174000"));
        assert!(is_uuid("123E4567-E89B-12D3-A456-426614174000"));
        assert!(!is_uuid("123e4567e89b12d3a456426614174000"));
        assert!(!is_uuid("{123e4567-e89b-12d3-a456-426614174000}"));
    }

    #[test]
    fn account_name_rules() {
        assert!(is_account_name("zhang_san"));
        assert!(is_account_name("abcd"));
        assert!(!is_account_name("abc")); // 太短
        assert!(!is_account_name("1abcd")); // 数字开头
        assert!(!is_account_name("zhang-san")); // 含连字符
        assert!(!is_account_name(&"a".repeat(33))); // 超 32 位
    }

    #[test]
    fn code_rules() {
        assert!(is_code("SYS_ADMIN"));
        assert!(is_code("001"));
        assert!(is_code("a.b-c"));
        assert!(!is_code(""));
        assert!(!is_code(&"a".repeat(51)));
    }

    #[test]
    fn chinese_name_rules() {
        assert!(is_chinese_name("张三"));
        assert!(is_chinese_name("阿依古丽·买买提"));
        assert!(!is_chinese_name("张"));
        assert!(!is_chinese_name("John Smith"));
    }

    #[test]
    fn password_rules() {
        assert!(is_password("Abcd1234"));
        assert!(is_password("abcd1234")); // 只要求含字母与数字，不强制大小写混用
        assert!(!is_password("abcdefgh"));
        assert!(!is_password("12345678"));
        assert!(!is_password("Abc 1234"));
        assert!(!is_password("Abc123"));
        assert!(!is_password("Abcd1234中文"));
    }

    #[test]
    fn ip_rules() {
        assert!(is_ip("10.0.0.1"));
        assert!(is_ip("::1"));
        assert!(is_ipv6("2001:db8::8a2e:370:7334"));
        assert!(!is_ipv4("010.1.1.1")); // 前导 0
        assert!(!is_ipv4("1.1.1"));
        assert!(!is_ip("[::1]"));
        assert!(!is_ip("example.com"));
    }
}
