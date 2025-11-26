Project name: Capi

What is Capi?
Capi is a local model runtime. It can run machine-learning models on the user’s own computer using Intel hardware acceleration (GPU, NPU or CPU). It offers a full CLI and a desktop interface. There is also a built-in chat window so users can interact with local models in a simple way. For developers or external tools, Capi exposes an OpenAI-compatible API.

The idea is to keep things simple and predictable. The UI lets users handle model downloads, conversion, selection, and settings without touching the CLI. The CLI exists for people who prefer automation or scripting. The backend service runs locally and is responsible for inference, model loading and device handling.

Objectives
- Provide a complete local inference service for Windows and Linux.
- Offer a full CLI and a complete UI.
- Provide a simple chat interface for everyday usage.
- Allow users to select installed models directly inside the chat UI.
- Expose a local OpenAI-compatible API (chat completions, embeddings, models).
- Detect available Intel hardware and pick the best device (or follow user preference).
- Provide a model search + download + conversion workflow for OpenVINO IR.
- Package installers for both platforms.

Architecture (high-level)
- Rust backend: inference engine, OpenVINO integration, model downloads, conversion, chat sessions, model registry, OpenAI-compatible API.
- CLI: uses backend logic directly.
- UI (Tauri): tray icon, chat window, model browser, settings.
- Local HTTP API: 
  - /v1/chat/completions  
  - /v1/completions  
  - /v1/embeddings  
  - /v1/models  
- Model registry: SQLite-based, storing metadata and installed models.
- Config directory: device preferences, auto-start, UI settings.

Reasoning behind structure
- Backend stays independent and reusable by both CLI and UI.
- The local API makes the UI simpler and also allows third-party tools to use Capi as a drop-in replacement for cloud inference.
- Chat UI is a natural companion for users who don’t want a terminal and just want to ask questions.

Out of scope (for now)
- Training or fine-tuning models.
- Remote inference over the network.
- Automatic installation of GPU/NPU drivers.
- Serving large numbers of concurrent clients.
- Alternative backends besides OpenVINO.

To research
- Support for online GGUF (https://blog.openvino.ai/blog-posts/openvino-genai-supports-gguf-models)

This document may evolve as the project grows.

