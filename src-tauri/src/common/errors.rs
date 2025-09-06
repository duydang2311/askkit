use std::{str::Utf8Error, time::SystemTimeError};

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
