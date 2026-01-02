//! Token management with automatic refresh.

use crate::auth::oauth::OAuth2Client;
use crate::error::{AppError, AuthError};
use crate::keychain;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time;
use tracing::info;

/// Message types for the token manager.
#[allow(dead_code)]
pub enum TokenMessage {
    /// Request to refresh the token now.
    RefreshNow,
    /// Signal to stop the token manager.
    Stop,
}

/// Manages access token lifecycle and automatic refresh.
#[allow(dead_code)]
pub struct TokenManager {
    oauth_client: Arc<OAuth2Client>,
    /// Sender for commands to the background task.
    command_tx: Option<mpsc::Sender<TokenMessage>>,
    /// Whether auto-refresh is currently active.
    is_running: Arc<Mutex<bool>>,
}

#[allow(dead_code)]
impl TokenManager {
    /// Create a new token manager.
    pub fn new(oauth_client: Arc<OAuth2Client>) -> Self {
        Self {
            oauth_client,
            command_tx: None,
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start automatic token refresh.
    ///
    /// This spawns a background task that refreshes the token before it expires.
    pub async fn start_auto_refresh(
        &mut self,
        expires_in_seconds: u64,
        refresh_before_seconds: u64,
        on_refresh: impl Fn(Result<(), AppError>) + Send + Sync + 'static,
    ) {
        // Stop any existing refresh task
        self.stop_auto_refresh().await;

        let (tx, mut rx) = mpsc::channel::<TokenMessage>(10);
        self.command_tx = Some(tx);

        let oauth_client = Arc::clone(&self.oauth_client);
        let is_running = Arc::clone(&self.is_running);
        let on_refresh = Arc::new(on_refresh);

        // Calculate when to refresh
        let refresh_in = expires_in_seconds.saturating_sub(refresh_before_seconds);

        *is_running.lock().await = true;

        tokio::spawn(async move {
            info!("Token auto-refresh scheduled in {} seconds", refresh_in);

            let mut interval = time::interval(time::Duration::from_secs(refresh_in.max(60)));
            let mut first_tick = true;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Skip the first immediate tick
                        if first_tick {
                            first_tick = false;
                            continue;
                        }

                        if !*is_running.lock().await {
                            break;
                        }

                        info!("Auto-refreshing token");
                        let result = refresh_token_internal(&oauth_client).await;
                        (on_refresh)(result);
                    }
                    Some(msg) = rx.recv() => {
                        match msg {
                            TokenMessage::RefreshNow => {
                                info!("Manual token refresh requested");
                                let result = refresh_token_internal(&oauth_client).await;
                                (on_refresh)(result);
                            }
                            TokenMessage::Stop => {
                                info!("Token auto-refresh stopped");
                                break;
                            }
                        }
                    }
                }
            }

            *is_running.lock().await = false;
        });
    }

    /// Stop automatic token refresh.
    pub async fn stop_auto_refresh(&mut self) {
        if let Some(tx) = self.command_tx.take() {
            let _ = tx.send(TokenMessage::Stop).await;
        }
        *self.is_running.lock().await = false;
    }

    /// Request an immediate token refresh.
    pub async fn refresh_now(&self) -> Result<(), AppError> {
        if let Some(tx) = &self.command_tx {
            tx.send(TokenMessage::RefreshNow).await.map_err(|_| {
                AppError::Auth(AuthError::TokenRefreshFailed("Channel closed".into()))
            })?;
            Ok(())
        } else {
            // No background task running, refresh directly
            refresh_token_internal(&self.oauth_client).await
        }
    }

    /// Check if auto-refresh is currently running.
    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }
}

/// Internal function to perform token refresh.
#[allow(dead_code)]
async fn refresh_token_internal(oauth_client: &OAuth2Client) -> Result<(), AppError> {
    // Get refresh token from keychain
    let refresh_token = keychain::get_refresh_token()?;

    // Exchange for new tokens
    let token_response = oauth_client.refresh_token(&refresh_token).await?;

    // Store new tokens
    keychain::store_access_token(&token_response.access_token)?;

    if let Some(new_refresh) = &token_response.refresh_token {
        keychain::store_refresh_token(new_refresh)?;
    }

    // Store new expiry time
    let expires_at = Utc::now() + Duration::seconds(token_response.expires_in as i64);
    keychain::store_token_expiry(&expires_at.to_rfc3339())?;

    info!("Token refreshed successfully, expires at {}", expires_at);

    Ok(())
}

/// Calculate the remaining time until token expiry.
#[allow(dead_code)]
pub fn time_until_expiry(expiry_str: &str) -> Option<Duration> {
    let expiry: DateTime<Utc> = expiry_str.parse().ok()?;
    let now = Utc::now();

    if expiry > now {
        Some(expiry - now)
    } else {
        None
    }
}

/// Format duration as human-readable string (e.g., "45 min", "1 hour").
pub fn format_duration(duration: Duration) -> String {
    let total_minutes = duration.num_minutes();

    if total_minutes < 1 {
        "< 1 min".to_string()
    } else if total_minutes < 60 {
        format!("{} min", total_minutes)
    } else {
        let hours = total_minutes / 60;
        let mins = total_minutes % 60;
        if mins == 0 {
            format!("{} hour{}", hours, if hours == 1 { "" } else { "s" })
        } else {
            format!("{}h {}m", hours, mins)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::seconds(30)), "< 1 min");
        assert_eq!(format_duration(Duration::minutes(5)), "5 min");
        assert_eq!(format_duration(Duration::minutes(45)), "45 min");
        assert_eq!(format_duration(Duration::hours(1)), "1 hour");
        assert_eq!(format_duration(Duration::hours(2)), "2 hours");
        assert_eq!(format_duration(Duration::minutes(90)), "1h 30m");
    }

    #[test]
    fn test_time_until_expiry() {
        let future = (Utc::now() + Duration::hours(1)).to_rfc3339();
        let duration = time_until_expiry(&future);
        assert!(duration.is_some());
        assert!(duration.unwrap().num_minutes() > 55);

        let past = (Utc::now() - Duration::hours(1)).to_rfc3339();
        let duration = time_until_expiry(&past);
        assert!(duration.is_none());
    }
}
