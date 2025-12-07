#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("运行时错误: {0}")]
    RuntimeError(String),
    #[error("运行时错误: {0}")]
    RuntimeXError(String, #[source] Box<dyn std::error::Error + Send + Sync>),
}
