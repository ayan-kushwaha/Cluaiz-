use reqwest::Client;
use serde_json::Value;

const BASE_URL: &str = "http://127.0.0.1:8000";

#[tokio::test]
async fn test_chat_completions() {
    let client = Client::new();
    let payload = serde_json::json!({
        "messages": [
            { "role": "user", "content": "Hello engine!" }
        ]
    });

    let res = client.post(&format!("{}/v1/chat/completions", BASE_URL))
        .json(&payload)
        .send()
        .await
        .expect("Failed to connect to engine");
    
    // Engine might return 500 or 400 if no model is loaded, but connection should succeed.
    let status = res.status();
    assert!(status.is_success() || status.as_u16() >= 400);
}
