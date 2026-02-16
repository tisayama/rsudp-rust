pub mod proto;
pub mod dedup;
pub mod publisher;
pub mod subscriber;

use google_cloud_pubsub::client::{Client, ClientConfig, google_cloud_auth};
use google_cloud_gax::conn::Environment;
use crate::settings::PubsubSettings;

/// Create an authenticated Pub/Sub client based on configuration and environment.
///
/// Resolution order:
/// 1. `PUBSUB_EMULATOR_HOST` env var → connect to emulator (no auth)
/// 2. `config.credentials_file` → use service account JSON
/// 3. `GOOGLE_APPLICATION_CREDENTIALS` env var → use service account JSON
/// 4. None → return error
pub async fn create_pubsub_client(config: &PubsubSettings) -> Result<Client, Box<dyn std::error::Error>> {
    // 1. Check for emulator
    if let Ok(emulator_host) = std::env::var("PUBSUB_EMULATOR_HOST") {
        if !emulator_host.is_empty() {
            tracing::info!("pubsub: Connecting to emulator at {}", emulator_host);
            let client_config = ClientConfig {
                environment: Environment::Emulator(emulator_host),
                project_id: Some(config.project_id.clone()),
                ..Default::default()
            };
            let client = Client::new(client_config).await?;
            return Ok(client);
        }
    }

    // 2. Check credentials_file in config
    let cred_path = if let Some(ref path) = config.credentials_file {
        if !path.is_empty() {
            Some(path.clone())
        } else {
            None
        }
    } else {
        None
    };

    // 3. Fall back to GOOGLE_APPLICATION_CREDENTIALS
    let cred_path = cred_path.or_else(|| std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok());

    match cred_path {
        Some(path) => {
            tracing::info!("pubsub: Authenticating with service account from {}", path);
            let cred = google_cloud_auth::credentials::CredentialsFile::new_from_file(path).await?;
            let client_config = ClientConfig::default().with_credentials(cred).await?;
            let client = Client::new(client_config).await?;
            Ok(client)
        }
        None => {
            Err("Pub/Sub credentials not configured. Set credentials_file in [pubsub] config, GOOGLE_APPLICATION_CREDENTIALS env var, or PUBSUB_EMULATOR_HOST for emulator.".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_client_missing_credentials_returns_error() {
        // Ensure no credentials env vars are set for this test
        std::env::remove_var("PUBSUB_EMULATOR_HOST");
        std::env::remove_var("GOOGLE_APPLICATION_CREDENTIALS");

        let config = PubsubSettings {
            enabled: true,
            project_id: "test-project".to_string(),
            credentials_file: None,
            ..Default::default()
        };

        let result = create_pubsub_client(&config).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("credentials"), "Error should mention credentials: {}", err_msg);
    }
}
