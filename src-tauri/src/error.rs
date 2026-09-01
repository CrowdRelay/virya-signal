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

impl AppError {
    /// Stable error category for client-side classification. The WebView
    /// parses this to decide whether a toast is transient, what timeout to
    /// use, and whether it should surface to the fan at all — without
    /// substring-matching the translated message text.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::NotConfigured => "not_configured",
            Self::InvalidPin => "invalid_pin",
            Self::Locked => "locked",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::InvalidInput(_) => "invalid_input",
            Self::Conflict(_) => "conflict",
            Self::NotFound => "not_found",
            Self::Remote { .. } => "remote",
            Self::Network(_) => "network",
            Self::Url(_) => "url",
            Self::Json(_) => "json",
            Self::Io(_) => "io",
            Self::StrongholdClient => "vault",
            Self::BackgroundTask => "background_task",
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        // Structured form: `{"kind": "...", "message": "..."}`. The WebView
        // bridge extracts both fields and embeds them in the error string as
        // `kind\x1fmessage` (unit separator), so `error_kind()` and
        // `error_message()` can split them without substring matching on the
        // translated message text.
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("kind", self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}
