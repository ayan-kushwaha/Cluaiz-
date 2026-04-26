# 📜 CURE Engine Coding Rules & Guidelines

These rules are strictly enforced for the `Cluaiz-ai-CURE` workspace to maintain a highly scalable, enterprise-grade architecture.

## 1. 📂 Strict Modular Folder Structure AVOID SINGLE-FILE DUMPING
- **Never** dump all logic (routing, handlers, logic, database connections) into a single `main.rs` or `lib.rs` file.
- **Always** break down logic into separate files and folders specific to their domains.
- Example: 
  - `api/src/main.rs` (Only App State and Server binding)
  - `api/src/routes/` (All endpoints)
  - `api/src/handlers/` (Logic for those endpoints)
  - `api/src/models/` (Structs and data formats)
  
## 2. 🧩 "One File, One Responsibility" (Separation of Concerns)
- Each file should only do **one thing**.
- If a file is crossing 150-200 lines, it is a clear indicator that it MUST be broken down into sub-modules.
- For example: Inside `storage/`, do not put Mongo, Neo4j, and MinIO logic in one file. They should be in `storage/src/mongo.rs`, `storage/src/neo4j.rs`, etc.

## 3. 🏗️ Scalable for 10 Million+ Users
- The architecture must be designed to scale effortlessly. This means modularity cannot be comprised.
- Every function, struct, and module must be properly named, documented, and placed in a dedicated, logical directory.

> **Note to AI**: ALWAYS review this file before generating or modifying code. Breaking these modularity guidelines is completely unacceptable.
