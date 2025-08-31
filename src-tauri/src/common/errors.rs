use serde::{ser::SerializeStruct, Serialize};

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

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        match self {
            AppError::HttpRequestBuilder(error) => {
                state.serialize_field("kind", "HttpRequestBuilder")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::HttpRedirect(error) => {
                state.serialize_field("kind", "HttpRedirect")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::HttpStatusCode(error) => {
                state.serialize_field("kind", "HttpStatusCode")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::HttpTimeout(error) => {
                state.serialize_field("kind", "HttpTimeout")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::HttpRequest(error) => {
                state.serialize_field("kind", "HttpRequest")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::HttpBody(error) => {
                state.serialize_field("kind", "HttpBody")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::HttpDecode(error) => {
                state.serialize_field("kind", "HttpDecode")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::Unknown(Some(error)) => {
                state.serialize_field("kind", "Unknown")?;
                state.serialize_field("message", &error.to_string())?;
            }
            AppError::Unknown(None) => {
                state.serialize_field("kind", "Unknown")?;
                state.serialize_field("message", "An unknown error occurred")?;
            }
        }
        state.end()
    }
}
