use jsonrpc_core::{Error, ErrorCode, Value};
use std::fmt;

mod codes {
    pub const ACCOUNT_ERROR: i64 = -32023;
    pub const ENCRYPTION_ERROR: i64 = -32055;
}

pub fn account<T: fmt::Debug>(error: &str, details: T) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::ACCOUNT_ERROR),
        message: error.into(),
        data: Some(Value::String(format!("{:?}", details))),
    }
}

pub fn encryption<T: fmt::Debug>(error: T) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::ENCRYPTION_ERROR),
        message: "Encryption error.".into(),
        data: Some(Value::String(format!("{:?}", error))),
    }
}

pub fn invalid_params<T: fmt::Debug>(param: &str, details: T) -> Error {
    Error {
        code: ErrorCode::InvalidParams,
        message: format!("Couldn't parse parameters: {}", param),
        data: Some(Value::String(format!("{:?}", details))),
    }
}
