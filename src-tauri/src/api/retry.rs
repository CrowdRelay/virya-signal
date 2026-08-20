use std::time::Duration;

use uuid::Uuid;

use crate::AppError;

pub(super) const GET_RETRY_DELAYS: &[Duration] = &[
    Duration::from_millis(200),
    Duration::from_millis(800),
    Duration::from_millis(2400),
];

fn jittered_delay(delay: Duration) -> Duration {
    let base_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX);
    if base_ms < 4 {
        return delay;
    }

    // Keep retries close to the existing backoff budget while preventing a
    // fleet of clients recovering from the same 429/5xx from retrying in lockstep.
    // ±25% is deliberately bounded so user-visible latency remains predictable.
    let spread_ms = base_ms / 4;
    let width = spread_ms.saturating_mul(2).saturating_add(1);
    let sample = (Uuid::new_v4().as_u128() as u64) % width;
    Duration::from_millis(base_ms.saturating_sub(spread_ms).saturating_add(sample))
}

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
            tokio::time::sleep(jittered_delay(*delay)).await;
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

pub(super) fn is_transient_failure(error: &AppError) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_jitter_stays_within_quarter_backoff_budget() {
        for delay in GET_RETRY_DELAYS {
            for _ in 0..64 {
                let jittered = jittered_delay(*delay);
                let lower = delay.mul_f64(0.75);
                let upper = delay.mul_f64(1.25) + Duration::from_millis(1);
                assert!(jittered >= lower, "{jittered:?} below {lower:?}");
                assert!(jittered <= upper, "{jittered:?} above {upper:?}");
            }
        }
    }
}
