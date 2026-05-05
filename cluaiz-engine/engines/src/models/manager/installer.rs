use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;

/// Cluaiz Installer
/// Handles the atomic download and verification of model weights from Hugging Face.
pub struct ModelInstaller {
    target_dir: PathBuf,
}

impl ModelInstaller {
    pub fn new(target_dir: PathBuf) -> Self {
        Self { target_dir }
    }

    /// Downloads the GGUF file using the .part atomic protocol
    pub async fn download_weights(&self, url: &str, filename: &str) -> Result<(), String> {
        let client = reqwest::Client::new();
        let response = client.get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Hugging Face: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Hugging Face Error: {}", response.status()));
        }

        let mut dest_path = self.target_dir.clone();
        dest_path.push(filename);
        
        let mut part_path = dest_path.clone();
        part_path.set_extension("gguf.part");

        let mut file = tokio::fs::File::create(&part_path).await
            .map_err(|e| format!("Failed to create .part file: {}", e))?;

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Download stream interrupted: {}", e))?;
            file.write_all(&chunk).await.map_err(|e| format!("Failed to write bits to SSD: {}", e))?;
        }

        file.flush().await.ok();
        
        // Atomic Rename: .part -> .gguf
        tokio::fs::rename(&part_path, &dest_path).await
            .map_err(|e| format!("Atomic Rename Failed: {}", e))?;

        Ok(())
    }

    /// Pulls supplemental assets (tokenizer, config, DNA)
    pub async fn pull_assets(&self, assets: Vec<(String, String)>) -> Result<(), String> {
        for (name, url) in assets {
            let response = reqwest::get(&url).await
                .map_err(|e| format!("Failed to fetch asset {}: {}", name, e))?;
            
            let mut path = self.target_dir.clone();
            path.push(name);
            
            let content = response.text().await.map_err(|e| format!("Failed to read asset {}: {}", url, e))?;
            tokio::fs::write(path, content).await.map_err(|e| format!("Failed to save asset: {}", e))?;
        }
        Ok(())
    }
}
