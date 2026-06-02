use crate::engine::OnnxEngine;
use neural_core::interfaces::router_contract::EngineError;
use ort::value::Value;
use ort::session::SessionOutputs;

impl OnnxEngine {
    pub fn execute_text_embedding(&self, text: &str) -> Result<Vec<f32>, EngineError> {
        let session = self.session.as_ref().ok_or_else(|| EngineError::Internal("Model not loaded".to_string()))?;
        let tokenizer = self.tokenizer.as_ref().ok_or_else(|| EngineError::Internal("Tokenizer not loaded".to_string()))?;

        // 1. Tokenize Text
        let encoding = tokenizer.encode(text, true).map_err(|e| EngineError::EmbeddingFailed(e.to_string()))?;
        let ids: Vec<i64> = encoding.get_ids().iter().map(|&x| x as i64).collect();
        let mask: Vec<i64> = encoding.get_attention_mask().iter().map(|&x| x as i64).collect();
        
        let batch_size = 1;
        let seq_len = ids.len();

        // 2. Create ORT Values using tuple `([shape], Vec<T>)` which implements `OwnedTensorArrayData`
        let ids_val = Value::from_array(([batch_size, seq_len], ids)).map_err(|_| EngineError::Internal("Bad Alloc".into()))?;
        let mask_val = Value::from_array(([batch_size, seq_len], mask)).map_err(|_| EngineError::Internal("Bad Alloc".into()))?;

        // 3. Run Inference (Microseconds on CPU)
        let inputs = ort::inputs![ids_val, mask_val];
        let mut locked_session = session.lock().map_err(|_| EngineError::Internal("Mutex Poised".into()))?;
        let outputs: SessionOutputs = locked_session.run(inputs).map_err(|e: ort::Error| EngineError::EmbeddingFailed(e.to_string()))?;

        // 4. Extract raw tensor and apply Mean Pooling
        let embeddings_tuple = outputs[0].try_extract_tensor::<f32>().map_err(|_| EngineError::Internal("Tensor Extract Failed".into()))?;
        
        let slice = embeddings_tuple.1;
        let hidden_dim = slice.len() / seq_len;
        
        // Manual Mean Pooling
        let mut vec = vec![0.0; hidden_dim];
        for token_idx in 0..seq_len {
            for dim in 0..hidden_dim {
                vec[dim] += slice[token_idx * hidden_dim + dim];
            }
        }
        for dim in 0..hidden_dim {
            vec[dim] /= seq_len as f32;
        }

        // L2 Normalization (Cosine Similarity Ready)
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        let normalized = vec.into_iter().map(|v| v / norm).collect();

        tracing::info!("⚡ [ONNX-Text] Vector generated successfully! (Dim: {})", seq_len);
        Ok(normalized)
    }
}
