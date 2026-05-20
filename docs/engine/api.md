# Axum API

The `cluaiz-engine` hosts a high-performance REST API gateway built on the Axum web framework and powered by a multithreaded Tokio runtime. 

---

## Endpoint Specifications

The gateway exposes clean HTTP and Server-Sent Event (SSE) interfaces to interface clients:

### 1. Node Health & Diagnostics
*   **`GET /`**: Server landing route displaying engine version metadata.
*   **`GET /health`**: Diagnostics endpoint returning active check status.
*   **`GET /info`**: Returns host system specifications and CPU core counts.
*   **`GET /status/embedded`**: Confirms host environment parameters.

### 2. Conversational Engine
*   **`POST /chat`**: Asynchronous generation endpoint. Stream outputs are sent to the client in real-time using **Server-Sent Events (SSE)**.
*   **`GET /history`**: Lists active chat session IDs and metadata configurations.
*   **`GET /history/{session_id}`**: Retrieves raw, chronological message buffers for a specific session.

### 3. Model Management & Telemetry
*   **`GET /models/available`**: Lists local cached models and available weights in the registry.
*   **`GET /hardware`**: Dynamic readout of GPU/NPU active memory limits and tensor engine loads.
*   **`POST /models/download`**: Spawns a background task to pull and cache weight tensors from remote mirrors.
*   **`POST /models/load`**: Dynamically allocates local RAM/VRAM to mount a specific model into active memory.
