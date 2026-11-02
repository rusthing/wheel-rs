//! # SM3 摘要算法
//!
//! 国密 SM3 密码杂凑算法（GB/T 32905-2016），输出 256 位（32 字节）摘要。
//!
//! ## 实现来源
//!
//! 算法本身用 [`sm3`] crate（RustCrypto 官方实现，与 [`sha2`] 共用 `digest`
//! 的 trait），本模块只做两件事：把输出固定成 `[u8; 32]`、给出小写十六进制串。
//!
//! 之所以还要这一层封装：本库与下游都用「字节摘要 + 小写 hex」这两种形态，
//! 直接在各处写 `Sm3::new()` / `hex::encode(...)` 会把 `digest` 的 trait 泄漏到
//! 每个调用点；密码学算法则不自己实现——手写的压缩函数既没有经过审计，
//! 也没有硬件加速路径。
//!
//! ## 与业务的关系
//!
//! `adm-rs` 的口令存的是 `SM3(小写(SM3(明文)) + 小写(密码组合码))`：
//! 前端先做一次（避免明文过网），服务端再做一次带组合码的外层摘要。
//!
//! ## 示例
//!
//! ```
//! use wheel_rs::sm3_utils::{sm3, sm3_hex};
//!
//! // 官方测试向量：SM3("abc")
//! assert_eq!(
//!     sm3_hex("abc"),
//!     "66c7f0f462eeedd9d1f2d46bdc10e4e24167c4875cf2f7a2297da02b8f4ba8e0"
//! );
//! // 输出定长 32 字节，与输入长度无关
//! assert_eq!(sm3("任意长度的输入".as_bytes()).len(), 32);
//! ```

use sm3::{Digest, Sm3};

/// # 计算 SM3 摘要
///
/// ## 参数
/// * `data` - 任意长度的输入字节
///
/// ## 返回值
/// * `[u8; 32]` - 32 字节摘要（大端）
///
/// ## 示例
/// ```
/// use wheel_rs::sm3_utils::sm3;
///
/// // 空输入的官方向量
/// let digest = sm3(b"");
/// assert_eq!(digest[0], 0x1a);
/// ```
pub fn sm3(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sm3::new();
    hasher.update(data);
    let digest = hasher.finalize();
    // 定长 32 字节，长度不会不匹配；用 copy 而不是依赖 hybrid-array 的转换，
    // 免得 digest 换底层数组类型时这里跟着改
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

/// # 计算 SM3 摘要的小写十六进制串
///
/// ## 参数
/// * `data` - 任意长度的输入字节；`&str` / `String` / `&[u8]` 都可以传
///
/// ## 返回值
/// * `String` - 64 个小写十六进制字符
///
/// ## 示例
/// ```
/// use wheel_rs::sm3_utils::sm3_hex;
///
/// assert_eq!(sm3_hex("abc").len(), 64);
/// assert_eq!(sm3_hex("abc"), sm3_hex("abc".as_bytes()));
/// ```
pub fn sm3_hex(data: impl AsRef<[u8]>) -> String {
    hex::encode(sm3(data.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 官方测试向量：空串
    #[test]
    fn empty_input() {
        assert_eq!(
            sm3_hex(""),
            "1ab21d8355cfa17f8e61194831e81a8f22bec8c728fefb747ed035eb5082aa2b"
        );
    }

    /// 官方测试向量："abc"
    #[test]
    fn abc() {
        assert_eq!(
            sm3_hex("abc"),
            "66c7f0f462eeedd9d1f2d46bdc10e4e24167c4875cf2f7a2297da02b8f4ba8e0"
        );
    }

    /// 官方测试向量："abcd" 重复 16 次（正好 64 字节，填充要新起一个分组）
    #[test]
    fn abcd_times_16() {
        assert_eq!(
            sm3_hex("abcd".repeat(16)),
            "debe9ff92275b8a138604889c18e5a4d6fdb70e5387e5765293dcba39c0c5732"
        );
    }

    /// 填充边界：55/56/64 字节落在「不新起分组」「新起分组」两侧
    #[test]
    fn padding_boundaries() {
        for len in [55, 56, 63, 64, 65] {
            assert_eq!(sm3(&vec![b'a'; len]).len(), 32);
        }
        assert_ne!(sm3(&[b'a'; 55]), sm3(&[b'a'; 56]));
    }

    /// 摘要对输入敏感：改一个字节结果完全不同
    #[test]
    fn avalanche() {
        assert_ne!(sm3(b"abcdabcdabcdabcd"), sm3(b"abcdabcdabcdabce"));
    }

    /// 一次摘要的字节序：返回的 32 字节与 hex 串是同一份数据
    #[test]
    fn bytes_match_hex() {
        let raw = sm3(b"abc");
        assert_eq!(hex::encode(raw), sm3_hex("abc"));
    }
}
