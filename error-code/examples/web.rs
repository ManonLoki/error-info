use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use backtrace::Backtrace;
use error_code::ToErrorInfo;
use error_code_derive::ToErrorInfo;
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{info, warn};

/// 应用错误
#[allow(dead_code)]
#[derive(Debug, Error, ToErrorInfo)]
#[error_info(app_type = "http::StatusCode", prefix = "0A")]
enum AppError {
    #[error("Invalid param: {0}")]
    #[error_info(code = "IP", app_code = "400")]
    InvalidParam(String),

    #[error("Item {0} not found")]
    #[error_info(code = "NF", app_code = "404")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    #[error_info(
        code = "ISE",
        app_code = "500",
        client_msg = "we had a server problem, please try again later"
    )]
    ServerError(String),

    #[error("Unknown error")]
    #[error_info(code = "UE", app_code = "500")]
    Unknown,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/", get(index_handler));

    let addr = "0.0.0.0:3000";
    info!("Listening on http://{}", addr);
    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

/// 处理Index
async fn index_handler() -> Result<&'static str, AppError> {
    let bt = Backtrace::new();
    Err(AppError::ServerError(format!("{bt:?}")))
}

/// 为Error实现Impl IntoResponse
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 在这里转换为ErrorInfo
        let info = self.to_error_info();
        let status = info.app_code;

        // 判断错误码是否为服务器错误
        if status.is_server_error() {
            warn!("{:?}", info);
        } else {
            info!("{:?}", info);
        }

        // 构建Response 使用的是ClientMessage
        Response::builder()
            .status(status)
            .body(info.to_string().into())
            .unwrap()
    }
}
