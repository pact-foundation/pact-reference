//! Utility functions

use std::panic::RefUnwindSafe;
use std::time::Duration;

use futures::StreamExt;
use pact_models::interaction::Interaction;
use reqwest::RequestBuilder;
use tokio::time::sleep;
use tracing::{trace, warn};

/// Returns true when a status code should trigger a retry.
///
/// Returns true for status codes that indicate a transient failure worth retrying:
/// 5xx server errors, 429 Too Many Requests, and 408 Request Timeout.
fn is_retryable(status: reqwest::StatusCode) -> bool {
  status.is_server_error()
    || status == reqwest::StatusCode::TOO_MANY_REQUESTS
    || status == reqwest::StatusCode::REQUEST_TIMEOUT
}

/// Compute the delay before the next retry.
///
/// For 429 with a `Retry-After` header: delay = header_secs + min(header_secs / 5, 60).
/// This adds ~20% on top (capped at 60 s extra) to spread retries across the new
/// rate-limit window instead of all hitting simultaneously.
///
/// For all other retryable statuses, and for 429 without the header: exponential
/// back-off of `500 × 2^(attempt − 1)` milliseconds, giving 500 ms, 1 s, 2 s, 4 s,
/// 8 s, … on successive retries.
pub(crate) fn compute_retry_delay(
  status: reqwest::StatusCode,
  retry_after: Option<std::time::Duration>,
  attempt: u32,
) -> Duration {
  if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
    if let Some(base) = retry_after {
      let secs = base.as_secs();
      let extra = (secs / 5).min(60);
      return Duration::from_secs(secs + extra);
    }
  }
  Duration::from_millis(500 * 2_u64.pow(attempt.saturating_sub(1)))
}

fn parse_retry_after(response: &reqwest::Response) -> Option<std::time::Duration> {
  let header_value = response
    .headers()
    .get(reqwest::header::RETRY_AFTER)?
    .to_str()
    .ok()?;

  // Try decimal-seconds form first (e.g. "120").
  if let Ok(secs) = header_value.trim().parse::<u64>() {
    return Some(std::time::Duration::from_secs(secs));
  }

  // Fall back to HTTP-date form (e.g. "Fri, 31 Dec 1999 23:59:59 GMT").
  if let Ok(system_time) = httpdate::parse_http_date(header_value) {
    let delay = system_time
      .duration_since(std::time::SystemTime::now())
      .unwrap_or_default();
    return Some(delay);
  }

  None
}

/// Retries a request on failure
pub(crate) async fn with_retries(retries: u8, request: RequestBuilder) -> Result<reqwest::Response, reqwest::Error> {
  match &request.try_clone() {
    None => {
      warn!("with_retries: Could not retry the request as it is not cloneable");
      request.send().await
    }
    Some(_) => {
      if retries == 0 {
        return request.send().await;
      }

      futures::stream::iter((1..=retries).step_by(1))
        .fold((None::<Result<reqwest::Response, reqwest::Error>>, request.try_clone()), |(response, request), attempt| {
          async move {
            match request {
              Some(request_builder) => match response {
                None => {
                  let next = request_builder.try_clone();
                  (Some(request_builder.send().await), next)
                },
                Some(response) => {
                  trace!("with_retries: attempt {}/{} is {:?}", attempt, retries, response);
                  match response {
                    Ok(ref res) => if is_retryable(res.status()) {
                      match request_builder.try_clone() {
                        None => (Some(response), None),
                        Some(rb) => {
                          let delay = compute_retry_delay(res.status(), parse_retry_after(res), attempt as u32);
                          sleep(delay).await;
                          (Some(request_builder.send().await), Some(rb))
                        }
                      }
                    } else {
                      (Some(response), None)
                    },
                    Err(ref err) => if err.is_status() {
                      if err.status().map(is_retryable).unwrap_or(false) {
                        match request_builder.try_clone() {
                          None => (Some(response), None),
                          Some(rb) => {
                            sleep(Duration::from_millis(10_u64.pow(attempt as u32))).await;
                            (Some(request_builder.send().await), Some(rb))
                          }
                        }
                      } else {
                        (Some(response), None)
                      }
                    } else {
                      (Some(response), None)
                    }
                  }
                }
              }
              None => (response, None)
            }
          }
        }).await.0.unwrap()
    }
  }
}

pub(crate) fn as_safe_ref(interaction: &dyn Interaction) -> Box<dyn Interaction + Send + Sync + RefUnwindSafe> {
  if let Some(v4) = interaction.as_v4_sync_message() {
    Box::new(v4)
  } else if let Some(v4) = interaction.as_v4_async_message() {
    Box::new(v4)
  } else {
    let v4 = interaction.as_v4_http().unwrap();
    Box::new(v4)
  }
}

#[cfg(test)]
mod tests {
  use std::sync::atomic::{AtomicUsize, Ordering};
  use std::sync::Arc;
  use std::time::Duration;

  use axum::response::IntoResponse;
  use axum::routing::get;
  use axum::Router;
  use tokio::net::TcpListener;

  use super::{compute_retry_delay, is_retryable, with_retries};

  // MARK: compute_retry_delay unit tests

  #[test]
  fn compute_retry_delay_429_without_retry_after_uses_exponential_backoff() {
    // attempt=2: 500 * 2^(2-1) = 1000 ms
    let delay = compute_retry_delay(reqwest::StatusCode::TOO_MANY_REQUESTS, None, 2);
    assert_eq!(delay, Duration::from_millis(1000));
  }

  #[test]
  fn compute_retry_delay_starts_at_500ms_on_first_attempt() {
    // attempt=1: 500 * 2^0 = 500 ms
    let delay = compute_retry_delay(reqwest::StatusCode::INTERNAL_SERVER_ERROR, None, 1);
    assert_eq!(delay, Duration::from_millis(500));
  }

  #[test]
  fn compute_retry_delay_429_with_retry_after_10_adds_20_percent() {
    // secs=10, extra=min(10/5, 60)=min(2,60)=2, total=12
    let delay = compute_retry_delay(reqwest::StatusCode::TOO_MANY_REQUESTS, Some(Duration::from_secs(10)), 1);
    assert_eq!(delay, Duration::from_secs(12));
  }

  #[test]
  fn compute_retry_delay_429_with_retry_after_400_caps_extra_at_60() {
    // secs=400, extra=min(400/5, 60)=min(80,60)=60, total=460
    let delay = compute_retry_delay(reqwest::StatusCode::TOO_MANY_REQUESTS, Some(Duration::from_secs(400)), 1);
    assert_eq!(delay, Duration::from_secs(460));
  }

  #[test]
  fn compute_retry_delay_429_with_retry_after_300_boundary_case() {
    // secs=300, extra=min(300/5, 60)=min(60,60)=60, total=360
    let delay = compute_retry_delay(reqwest::StatusCode::TOO_MANY_REQUESTS, Some(Duration::from_secs(300)), 1);
    assert_eq!(delay, Duration::from_secs(360));
  }

  #[test]
  fn compute_retry_delay_5xx_ignores_retry_after_uses_exponential_backoff() {
    // attempt=3: 500 * 2^(3-1) = 2000 ms; Retry-After header should be ignored for 5xx
    let delay = compute_retry_delay(reqwest::StatusCode::INTERNAL_SERVER_ERROR, Some(Duration::from_secs(99999)), 3);
    assert_eq!(delay, Duration::from_millis(2000));
  }

  // MARK: is_retryable unit tests

  #[test]
  fn is_retryable_returns_true_for_500() {
    assert!(is_retryable(reqwest::StatusCode::INTERNAL_SERVER_ERROR));
  }

  #[test]
  fn is_retryable_returns_true_for_429() {
    assert!(is_retryable(reqwest::StatusCode::TOO_MANY_REQUESTS));
  }

  #[test]
  fn is_retryable_returns_true_for_408() {
    assert!(is_retryable(reqwest::StatusCode::REQUEST_TIMEOUT));
  }

  #[test]
  fn is_retryable_returns_false_for_404() {
    assert!(!is_retryable(reqwest::StatusCode::NOT_FOUND));
  }

  #[test]
  fn is_retryable_returns_false_for_200() {
    assert!(!is_retryable(reqwest::StatusCode::OK));
  }

  // MARK: with_retries integration tests

  /// Spawn a minimal axum server. The handler calls the provided closure on each request,
  /// passing the zero-based call count, and increments the shared call counter.
  async fn spawn_server<F>(handler: F) -> (String, Arc<AtomicUsize>)
  where
    F: Fn(usize) -> axum::response::Response + Send + Sync + 'static,
  {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    let handler = Arc::new(handler);

    type HandlerState = (Arc<AtomicUsize>, Arc<dyn Fn(usize) -> axum::response::Response + Send + Sync>);

    let app = Router::new()
      .route(
        "/",
        get(|axum::extract::State(state): axum::extract::State<HandlerState>| async move {
          let (ctr, h) = state;
          let n = ctr.fetch_add(1, Ordering::SeqCst);
          h(n)
        }),
      )
      .with_state((counter_clone, handler as Arc<dyn Fn(usize) -> axum::response::Response + Send + Sync>));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
      axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{}/", addr), counter)
  }

  #[tokio::test]
  async fn with_retries_retries_429_responses() {
    let (url, call_count) = spawn_server(|n| {
      if n < 2 {
        axum::http::StatusCode::TOO_MANY_REQUESTS.into_response()
      } else {
        axum::http::StatusCode::OK.into_response()
      }
    })
    .await;

    let client = reqwest::Client::new();
    let request = client.get(&url);
    // retries=3: attempt 1 → 429, attempt 2 → 429, attempt 3 → 200
    let result = with_retries(3, request).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status(), 200);
    assert_eq!(call_count.load(Ordering::SeqCst), 3);
  }

  #[tokio::test]
  async fn with_retries_does_not_retry_404_responses() {
    let (url, call_count) = spawn_server(|_| axum::http::StatusCode::NOT_FOUND.into_response()).await;

    let client = reqwest::Client::new();
    let request = client.get(&url);
    let result = with_retries(3, request).await;
    // 404 is not retryable — should get a response after exactly 1 call
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status(), 404);
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
  }

  #[tokio::test]
  async fn with_retries_zero_retries_sends_exactly_once_without_panic() {
    let (url, call_count) =
      spawn_server(|_| axum::http::StatusCode::TOO_MANY_REQUESTS.into_response()).await;

    let client = reqwest::Client::new();
    let request = client.get(&url);
    // retries=0 must not panic and must send the request exactly once
    let result = with_retries(0, request).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status(), 429);
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
  }

  #[tokio::test]
  async fn http_date_retry_after_is_parsed() {
    // Epoch (Jan 1 1970) is in the past → duration_since(now) underflows → unwrap_or_default() → 0 s delay.
    // The important thing is that the HTTP-date form is accepted and the retry proceeds.
    let (url, call_count) = spawn_server(|n| {
      if n < 1 {
        (
          axum::http::StatusCode::TOO_MANY_REQUESTS,
          [(axum::http::header::RETRY_AFTER, "Thu, 01 Jan 1970 00:00:00 GMT")],
        )
          .into_response()
      } else {
        axum::http::StatusCode::OK.into_response()
      }
    })
    .await;

    let client = reqwest::Client::new();
    let request = client.get(&url);
    let result = with_retries(3, request).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status(), 200);
    assert_eq!(call_count.load(Ordering::SeqCst), 2);
  }
}
