use error_code::*;
use error_code_derive::ToErrorInfo;

#[derive(Debug, thiserror::Error, ToErrorInfo)]
#[error_info(app_type = "http::StatusCode", prefix = "01")]
pub enum MyError {
    #[error("Invalid command: {0}")]
    #[error_info(code = "IC", app_code = "400")]
    InvalidCommand(String),
}

fn main() {
    let err = MyError::InvalidCommand("cmd".to_string());
    let info = err.to_error_info();
    println!("{:?}", info);
}

// 生成结果
// use error_code::{ErrorInfo, ToErrorInfo as _};
// impl ToErrorInfo for MyError {
//     type T = http::StatusCode;
//     fn to_error_info(&self) -> ErrorInfo<Self::T> {
//         match self {
//             MyError::InvalidCommand(_) => ErrorInfo::new("400", "01IC", "", self),
//         }
//     }
// }
