//! Discord webhook alerts.
//!
//! Messages are sent in the background so the bot loop never waits on the network,
//! except for `send_discord_blocking`, which is used right before the app exits.

use std::time::Duration;

use tauri::api::http::{Body, ClientBuilder, HttpRequestBuilder};

const USERNAME: &str = "Neuz";

async fn post(url: String, content: String) -> Result<(), String> {
    let client = ClientBuilder::new()
        .connect_timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
    let request = HttpRequestBuilder::new("POST", url)
        .map_err(|e| e.to_string())?
        .timeout(Duration::from_secs(5))
        .body(Body::Json(serde_json::json!({
            "username": USERNAME,
            "content": content,
        })));
    client.send(request).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Send a Discord message without blocking the caller. Does nothing when `url` is `None`.
pub fn send_discord(url: Option<String>, content: impl Into<String>) {
    if let Some(url) = url {
        let content = content.into();
        tauri::async_runtime::spawn(async move {
            let _ = post(url, content).await;
        });
    }
}

/// Send a Discord message and wait (at most ~5s) for it to go out.
/// Use this right before the app exits, otherwise the message would be lost.
pub fn send_discord_blocking(url: Option<String>, content: impl Into<String>) {
    if let Some(url) = url {
        let content = content.into();
        let _ = tauri::async_runtime::block_on(post(url, content));
    }
}
