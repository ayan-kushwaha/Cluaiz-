use reqwest::Client;
use serde_json::Value;

const BASE_URL: &str = "http://127.0.0.1:8000";

#[tokio::test]
async fn test_list_skills() {
    let client = Client::new();
    let res = client.get(&format!("{}/v1/skills/list", BASE_URL))
        .send()
        .await
        .expect("Failed to connect to engine");
    
    assert!(res.status().is_success());
}

#[tokio::test]
async fn test_hardware_status() {
    let client = Client::new();
    let res = client.get(&format!("{}/hardware", BASE_URL))
        .send()
        .await
        .expect("Failed to connect to engine");
    
    assert!(res.status().is_success());
}
