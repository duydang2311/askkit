use std::{str::Utf8Error, string::FromUtf8Error, time::SystemTimeError};

use serde::{ser::SerializeStruct, Serialize, Serializer};
use serde_json::error::Category;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("HTTP request builder error")]
    HttpRequestBuilder(reqwest::Error),
    #[error("HTTP redirect error")]
    HttpRedirect(reqwest::Error),
    #[error("HTTP status code error")]
    HttpStatusCode(reqwest::Error),
    #[error("HTTP timeout error")]
    HttpTimeout(reqwest::Error),
    #[error("HTTP request error")]
    HttpRequest(reqwest::Error),
    #[error("HTTP body error")]
    HttpBody(reqwest::Error),
    #[error("HTTP decode error")]
    HttpDecode(reqwest::Error),
    #[error("UTF8 parsing error")]
    Utf8(Utf8Error),
    #[error("Json error")]
    Json(serde_json::Error),
    #[error("System time error")]
    SystemTime(SystemTimeError),
    #[error("Keyring error")]
    Keyring(keyring::Error),
    #[error("AEAD error")]
    AesGcm(aes_gcm::Error),
    #[error("From UTF8 error")]
    FromUtf8(FromUtf8Error),
    #[error("Sqlx error: {0}")]
    Sqlx(sqlx::Error),
    #[error("Invalid agent provider error")]
    InvalidAgentProvider(String),
    #[error("Agent required error")]
    AgentRequired,
    #[error("Agent text gen params required error")]
    AgentTextGenParamsRequired,
    #[error("Mutex try lock error: {0}")]
    TryLock(tokio::sync::TryLockError),
    #[error("Transaction is still in use")]
    TransactionInUse,
    #[error("Unknown error")]
    Unknown(Option<Box<dyn std::error::Error + Send + Sync>>),
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        if value.is_builder() {
            AppError::HttpRequestBuilder(value)
        } else if value.is_redirect() {
            AppError::HttpRedirect(value)
        } else if value.is_status() {
            AppError::HttpStatusCode(value)
        } else if value.is_timeout() {
            AppError::HttpTimeout(value)
        } else if value.is_request() {
            AppError::HttpRequest(value)
        } else if value.is_body() {
            AppError::HttpBody(value)
        } else if value.is_decode() {
            AppError::HttpDecode(value)
        } else {
            AppError::Unknown(Some(Box::new(value)))
        }
    }
}

impl From<Utf8Error> for AppError {
    fn from(value: Utf8Error) -> Self {
        AppError::Utf8(value)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        AppError::Json(value)
    }
}

impl From<SystemTimeError> for AppError {
    fn from(value: SystemTimeError) -> Self {
        AppError::SystemTime(value)
    }
}

impl From<keyring::Error> for AppError {
    fn from(value: keyring::Error) -> Self {
        AppError::Keyring(value)
    }
}

impl From<aes_gcm::Error> for AppError {
    fn from(value: aes_gcm::Error) -> Self {
        AppError::AesGcm(value)
    }
}

impl From<FromUtf8Error> for AppError {
    fn from(value: FromUtf8Error) -> Self {
        AppError::FromUtf8(value)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        AppError::Sqlx(value)
    }
}

impl From<tokio::sync::TryLockError> for AppError {
    fn from(value: tokio::sync::TryLockError) -> Self {
        AppError::TryLock(value)
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state: <S as Serializer>::SerializeStruct;
        match self {
            AppError::HttpRequestBuilder(error) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "HttpRequestBuilder")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::HttpRedirect(error) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "HttpRedirect")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::HttpStatusCode(error) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "HttpStatusCode")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::HttpTimeout(error) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "HttpTimeout")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::HttpRequest(error) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "HttpRequest")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::HttpBody(error) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "HttpBody")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::HttpDecode(error) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "HttpDecode")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::Utf8(error) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "Utf8Error")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::Json(error) => {
                state = serializer.serialize_struct("AppError", 3)?;
                state.serialize_field("kind", "JsonError")?;
                state.serialize_field(
                    "category",
                    match error.classify() {
                        Category::Io => "io",
                        Category::Syntax => "syntax",
                        Category::Data => "data",
                        Category::Eof => "eof",
                    },
                )?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::SystemTime(error) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "SystemTimeError")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::Keyring(error) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "KeyringError")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::AesGcm(error) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "AesGcmError")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::FromUtf8(error) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "FromUtf8Error")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::Sqlx(error) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "SqlxError")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::InvalidAgentProvider(provider) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "InvalidAgentProviderError")?;
                state
                    .serialize_field("message", &format!("invalid agent provider: {}", provider))?;
            }
            AppError::AgentRequired => {
                state = serializer.serialize_struct("AppError", 1)?;
                state.serialize_field("kind", "AgentRequiredError")?;
            }
            AppError::AgentTextGenParamsRequired => {
                state = serializer.serialize_struct("AppError", 1)?;
                state.serialize_field("kind", "AgentTextGenParamsRequiredError")?;
            }
            AppError::TryLock(error) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "TryLockError")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::TransactionInUse => {
                state = serializer.serialize_struct("AppError", 1)?;
                state.serialize_field("kind", "TransactionInUse")?;
            }
            AppError::Unknown(Some(error)) => {
                state = serializer.serialize_struct("AppError", 2)?;
                state.serialize_field("kind", "Unknown")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::Unknown(None) => {
                state = serializer.serialize_struct("AppError", 1)?;
                state.serialize_field("kind", "Unknown")?;
            }
        }
        state.end()
    }
}
