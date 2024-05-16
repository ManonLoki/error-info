use core::fmt;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    str::FromStr,
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

/// ErrorInfo
pub struct ErrorInfo<T> {
    /// App Code 对应的是类型
    pub app_code: T,
    /// 错误码
    pub code: &'static str,
    /// Hash
    pub hash: String,
    /// 客户端消息
    pub client_msg: &'static str,
    /// 服务端消息
    pub server_msg: String,
}

/// ToErrorInfo
pub trait ToErrorInfo {
    /// 关联输出
    type T: FromStr;
    /// 转换为ErrorInfo
    fn to_error_info(&self) -> ErrorInfo<Self::T>;
}

impl<T> ErrorInfo<T>
where
    T: FromStr,
    <T as FromStr>::Err: fmt::Debug, // FromStr 还要实现Debug
{
    /// 新建
    pub fn new(
        app_code: &str,
        code: &'static str,
        client_msg: &'static str,
        server_msg: impl fmt::Display,
    ) -> Self {
        // 将server_msg 转换为字符串
        let server_msg = server_msg.to_string();
        // 计算ServerMsg的哈希
        let mut hasher = DefaultHasher::new();
        server_msg.hash(&mut hasher);
        let hash = hasher.finish();
        let hash = URL_SAFE_NO_PAD.encode(hash.to_be_bytes());

        Self {
            // 这个用FromStr进行转换 将原始类型转换为字符串
            app_code: T::from_str(app_code).expect("Can not parse app_code"),
            code,
            hash,
            client_msg,
            server_msg,
        }
    }
}

/// 实现 Display Trait
impl<T> fmt::Display for ErrorInfo<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}-{}] {}", self.code, self.hash, self.client_msg)
    }
}

/// 实现 Debug Trait
impl<T> fmt::Debug for ErrorInfo<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}-{}] {}", self.code, self.hash, self.server_msg)
    }
}

// 转换结果
// use error_code::{ErrorInfo, ToErrorInfo as _};
// impl ToErrorInfo for MyError {
//     type T = http::StatusCode;
//     fn to_error_info(&self) -> ErrorInfo<Self::T> {
//         match self {
//             MyError::InvalidCommand(_) => ErrorInfo::new("400", "01IC", "", self),
//             MyError::InvalidArgument(_) => ErrorInfo::new("400", "01IA", "friendly msg", self),
//             MyError::RespError(_) => ErrorInfo::new("500", "01RE", "", self),
//         }
//     }
// }
