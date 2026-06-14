use reqwest::Client;
use serde_json::Value;

const BASE_URL: &str = "http://127.0.0.1:8000";

#[tokio::test]
async fn test_db_execute_cdql() {
    let client = Client::new();
    // Test a basic CDQL query
    let payload = serde_json::json!({
        "query": "find Neuron(id: 'test_node')"
    });

    let res = client.post(&format!("{}/v1/db/execute", BASE_URL))
        .json(&payload)
        .send()
        .await
        .expect("Failed to connect to engine. Is it running on port 8000?");
    
    // As long as it responds successfully (even if neuron isn't found, it returns success)
    assert!(res.status().is_success());
}
