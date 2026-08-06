use std::fmt;

use crate::i18n::tr;

#[derive(Debug)]
pub enum AppError {
    NotConfigured,
    InvalidPin,
    Locked,
    Unauthorized,
    Forbidden,
    InvalidInput(String),
    Conflict(String),
    NotFound,
    Remote { status: u16, detail: String },
    Network(reqwest::Error),
    Url(url::ParseError),
    Json(serde_json::Error),
    Io(std::io::Error),
    StrongholdClient,
    BackgroundTask,
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConfigured => formatter.write_str(tr("native_error_not_configured")),
            Self::InvalidPin => formatter.write_str(tr("native_error_invalid_pin")),
            Self::Locked => formatter.write_str(tr("native_error_locked")),
            Self::Unauthorized => formatter.write_str(tr("native_error_unauthorized")),
            Self::Forbidden => formatter.write_str(tr("native_error_forbidden")),
            Self::InvalidInput(detail) => formatter.write_str(detail),
            Self::Conflict(detail) => {
                write!(formatter, "{}: {detail}", tr("native_error_conflict"))
            }
            Self::NotFound => formatter.write_str(tr("native_error_not_found")),
            Self::Remote { status, detail } => write!(
                formatter,
                "{} HTTP {status}: {detail}",
                tr("native_error_crowdrelay")
            ),
            Self::Network(error) => {
                write!(formatter, "{}: {error}", tr("native_error_network"))
            }
            Self::Url(error) => write!(formatter, "{}: {error}", tr("native_error_url")),
            Self::Json(error) => write!(formatter, "{}: {error}", tr("native_error_data")),
            Self::Io(error) => write!(formatter, "{}: {error}", tr("native_error_file")),
            Self::StrongholdClient => formatter.write_str(tr("native_error_vault")),
            Self::BackgroundTask => formatter.write_str(tr("native_error_background_task")),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Network(error) => Some(error),
            Self::Url(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        Self::Network(error)
    }
}

impl From<url::ParseError> for AppError {
    fn from(error: url::ParseError) -> Self {
        Self::Url(error)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
