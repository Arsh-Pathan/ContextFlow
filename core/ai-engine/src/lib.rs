//! ContextFlow AI engine.
//!
//! Hosts the `AiProvider` trait (OpenAI / Anthropic / Gemini / Ollama
//! implementations), the streaming cleanup pipeline that removes filler
//! words and resolves spoken corrections, and the voice-command transformer.
//!
//! ## Status
//!
//! Implemented in Slice 4.
