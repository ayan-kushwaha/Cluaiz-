//! 🌐 Remote Bridge: Network-based client connection to central cluaizd daemon.
//! Leverages HTTP transport to synchronize neuron states across clustered nodes.

use super::storage_bridge::CognitiveStorageBridge;
use std::time::Duration;
use reqwest::blocking::Client;
use serde_json::json;

pub struct RemoteBridge {
    client: Client,
    base_url: String,
}

impl RemoteBridge {
    pub fn new(addr: &str) -> Self {
        // Prepend http:// if not present
        let mut base_url = addr.trim().to_string();
        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
            base_url = format!("http://{}", base_url);
        }

        // Initialize connection pooling with a strict timeout of 1.5s
        let client = Client::builder()
            .timeout(Duration::from_millis(1500))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_default();

        RemoteBridge { client, base_url }
    }

    fn execute_with_retry<F, T>(&self, mut operation: F) -> Option<T>
    where
        F: FnMut() -> anyhow::Result<T>,
    {
        let mut retries = 3;
        while retries > 0 {
            match operation() {
                Ok(res) => return Some(res),
                Err(e) => {
                    tracing::warn!("Remote database query failed ({} retries left): {}", retries - 1, e);
                    retries -= 1;
                    if retries > 0 {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        }
        None
    }
}

impl CognitiveStorageBridge for RemoteBridge {
    fn inject_context(&self, memory_key: &str) -> Option<Vec<u8>> {
        // 1. Generate 16-byte UUID from the memory_key string
        let mut id_array = [0u8; 16];
        let key_bytes = memory_key.as_bytes();
        for (i, &b) in key_bytes.iter().take(16).enumerate() {
            id_array[i] = b;
        }
        let uuid = uuid::Uuid::from_bytes(id_array);
        let uuid_str = uuid.to_string();

        let url = format!("{}/neuron/{}", self.base_url, uuid_str);
        
        // 2. Perform GET query with retry strategy
        let response_body = self.execute_with_retry(|| {
            let res = self.client.get(&url)
                .header("x-tenant-id", "default_sandbox")
                .send()?;
            if res.status().is_success() {
                Ok(res.text()?)
            } else {
                anyhow::bail!("HTTP status {}", res.status())
            }
        })?;

        // 3. Parse and extract payload bytes
        let json_val: serde_json::Value = serde_json::from_str(&response_body).ok()?;
        if let Some(payload_val) = json_val.get("raw_payload") {
            if let Ok(bytes) = serde_json::from_value::<Vec<u8>>(payload_val.clone()) {
                tracing::info!("🧠 Successfully injected Remote Context: {} bytes", bytes.len());
                return Some(bytes);
            }
        }
        
        None
    }

    fn save_context(&self, memory_id: &str, payload: &str, vector: [f32; 16]) -> Result<(), String> {
        let url = format!("{}/neuron", self.base_url);
        
        // Create dummy SHA-256 creator hash for compatibility
        let creator_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string();
        
        let body = json!({
            "raw_payload": payload,
            "vector_data": vector,
            "model_creator_hash": creator_hash,
            "payload_type": "text",
            "dna": null,
            "adjacency": null
        });

        let write_success = self.execute_with_retry(|| {
            let res = self.client.post(&url)
                .header("x-tenant-id", "default_sandbox")
                .json(&body)
                .send()?;
            if res.status().is_success() {
                Ok(())
            } else {
                anyhow::bail!("HTTP status {}", res.status())
            }
        });

        match write_success {
            Some(_) => Ok(()),
            None => Err("Failed to execute remote network database write after retries".to_string()),
        }
    }
}
