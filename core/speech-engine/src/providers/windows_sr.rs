//! Windows.Media.SpeechRecognition provider — Slice 1 target.
//!
//! This file is intentionally empty pending the real WinRT implementation,
//! which lands once the local toolchain (MSVC Build Tools + Windows SDK) is
//! installed and we can compile and link against `windows::Media::SpeechRecognition`.
//!
//! Tracking: Slice 1, see `ROADMAP.md`.
//!
//! Until then, no `WindowsSpeechProvider` type is exported — the module is a
//! placeholder for the file path so future commits land in the expected
//! location. Importing this module is a compile error by design: nothing in
//! the rest of the workspace may depend on a provider that does not yet
//! exist.
