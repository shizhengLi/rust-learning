use thiserror::Error;

pub type Result<T> = std::result::Result<T, DatabaseError>;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("表 '{0}' 已存在")]
    TableExists(String),

    #[error("表 '{0}' 不存在")]
    TableNotFound(String),

    #[error("列 '{0}' 不存在")]
    ColumnNotFound(String),

    #[error("数据类型不匹配: {0}")]
    TypeMismatch(String),

    #[error("违反唯一约束: {0}")]
    UniqueViolation(String),

    #[error("违反非空约束: {0}")]
    NotNullViolation(String),

    #[error("解析错误: {0}")]
    ParseError(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON 错误: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("其他错误: {0}")]
    Other(String),
}

impl DatabaseError {
    pub fn column_not_found<S: Into<String>>(column: S) -> Self {
        Self::ColumnNotFound(column.into())
    }

    pub fn type_mismatch<S: Into<String>>(msg: S) -> Self {
        Self::TypeMismatch(msg.into())
    }

    pub fn unique_violation<S: Into<String>>(msg: S) -> Self {
        Self::UniqueViolation(msg.into())
    }

    pub fn not_null_violation<S: Into<String>>(msg: S) -> Self {
        Self::NotNullViolation(msg.into())
    }

    pub fn parse_error<S: Into<String>>(msg: S) -> Self {
        Self::ParseError(msg.into())
    }

    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::Other(msg.into())
    }
}