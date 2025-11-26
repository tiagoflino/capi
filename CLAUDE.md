Capi — development guideline

Purpose
These notes exist to guide development. Since Capi now has both a CLI, a UI, a chat interface and an OpenAI-compatible API, it is helpful to keep structure clear and predictable.

Core architecture
- Backend is a Rust library + a small server executable.
- CLI is a thin binary calling backend functions directly.
- UI (Tauri) talks to the backend through the local API.
- Chat UI uses the same local API that external tools will use.
- Backend exposes an OpenAI-compatible API so external tools can use Capi without custom configuration.

Backend responsibilities
- Detect hardware (GPU, NPU, CPU) and load models accordingly.
- Maintain the active model context.
- Provide an inference loop (token-by-token).
- Manage a simple chat memory per session.
- Serve OpenAI-compatible endpoints.
- Handle model listing, download, conversion, metadata and unloading.
- Keep a local registry and basic logs.

Rust practices
- Put OpenVINO integration in a module named something like openvino_engine.
- Only the FFI boundary uses unsafe. Wrap everything else.
- Model loading should always return Result with detailed errors.
- Long operations (downloads, conversions, large model loads) should run on background tasks.
- Maintain one inference worker per active model. Avoid blocking main thread.
- Keep API handlers thin; delegate logic to modules.

OpenAI-compatible API
- Expose /v1/chat/completions, /v1/completions, /v1/embeddings, /v1/models.
- Match request and response formats closely.
- For model selection, support both: 
  a) selecting model inside request 
  b) using the currently active model
- For streaming responses, support SSE (Server-Sent Events).
- For embeddings, load a dedicated embedding model if required.

Chat UI implementation
- The UI shows:
  - model selector (drop-down with installed models)
  - input box
  - message history
  - stop generation button
- The UI calls the local OpenAI-compatible API directly.
- Store chat history on disk in a simple structure.
- Include basic keyboard shortcuts.

Model management UI
- Show installed models.
- Show metadata (size, quantization, context).
- Allow installing and removing models.
- Expose search for remote models (Hugging Face), and trigger downloads and conversion.

Settings UI
- Auto-start toggle.
- Device preference order.
- Backend port.
- Memory cache settings.
- Folder locations.

CLI guidelines
- Provide subcommands: serve, model (list, download, convert, remove), config, generate.
- Generate should use the backend directly, not the OpenAI HTTP API.
- Keep error messages human-readable.
- Use structured output when useful (e.g., JSON for scripting).

Testing
- Test OpenAI-compatible API endpoints with small models.
- Test chat session handling, especially with streaming.
- Test UI → backend communication manually.
- Verify that GPU/NPU fallback works on real hardware.

Limitations to remember
- Some models will fail during conversion.
- Hardware differences between machines may cause unexpected performance issues.
- Model loading can be slow depending on hardware and format.

Repository organization
- Only update or create new documentation if explicitly requested
- Use comments in code files only if strictly necessary
- Use direct an plain english language. Keep it to a minimal and don't use emojis or AI-like language and formating.
- If you need to refactor, go to Plan mode always. Never change architecture or project structure without asking first
- To keep the code clean, we will always use replacement refactor without preserving compatibility with old code versions
- KEEP code and files CLEAN and readable

Final notes
Capi should feel simple and calm. The UI should be usable by people who do not know how to use a CLI. The CLI should be fully capable for those who prefer automation. The OpenAI API should allow external editors and tools to work with Capi with minimal configuration.

