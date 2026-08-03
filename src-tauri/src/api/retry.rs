use std::time::Duration;

use crate::AppError;

pub(super) const GET_RETRY_DELAYS: &[Duration] = &[
    Duration::from_millis(200),
    Duration::from_millis(800),
    Duration::from_millis(2400),
];

pub(super) async fn retry_idempotent<T, F, Fut>(attempt: F) -> Result<T, AppError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
{
    let mut last_error = None;
    for (index, delay) in std::iter::once(&Duration::ZERO)
        .chain(GET_RETRY_DELAYS.iter())
        .enumerate()
    {
        if index > 0 {
            tokio::time::sleep(*delay).await;
        }
        match attempt().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                if !is_transient_failure(&error) {
                    return Err(error);
                }
                eprintln!(
                    "[virya:retry] transient failure (attempt {}): {error}",
                    index + 1
                );
                last_error = Some(error);
            }
        }
    }
    Err(last_error.unwrap_or(AppError::BackgroundTask))
}

fn is_transient_failure(error: &AppError) -> bool {
    match error {
        AppError::Network(reqwest_error) => {
            reqwest_error.is_timeout() || reqwest_error.is_connect() || reqwest_error.is_request()
        }
        AppError::Remote { status, .. } => {
            matches!(*status, 408 | 425 | 429 | 500 | 502 | 503 | 504)
        }
        _ => false,
    }
}
