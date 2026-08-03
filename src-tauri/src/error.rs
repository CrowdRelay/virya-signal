use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Profil urządzenia nie jest skonfigurowany")]
    NotConfigured,
    #[error("Nieprawidłowy PIN")]
    InvalidPin,
    #[error("Sesja jest zablokowana")]
    Locked,
    #[error("Token urządzenia jest nieprawidłowy albo nie ma wymaganych uprawnień")]
    Unauthorized,
    #[error("Ta operacja wymaga roli owner")]
    Forbidden,
    #[error("{0}")]
    InvalidInput(String),
    #[error("Konflikt: {0}")]
    Conflict(String),
    #[error("Nie znaleziono danych")]
    NotFound,
    #[error("CrowdRelay HTTP {status}: {detail}")]
    Remote { status: u16, detail: String },
    #[error("Błąd sieci: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Błędny URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("Błąd danych: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Błąd pliku: {0}")]
    Io(#[from] std::io::Error),
    #[error("Błąd magazynu sejfu")]
    StrongholdClient,
    #[error("Wewnętrzny błąd zadania")]
    BackgroundTask,
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
