//! ActionEngine - V8-based script execution engine for Auth9 Actions
//!
//! This module provides the core engine for executing TypeScript/JavaScript actions
//! in a secure V8 isolate sandbox. Key features:
//!
//! - **V8 Isolate Sandbox**: Each script runs in an isolated V8 instance
//! - **Async/Await Support**: Scripts can use `async/await`, `fetch()`, `setTimeout`
//! - **TypeScript Support**: Automatic transpilation to JavaScript
//! - **Timeout Control**: Enforced execution timeout per action
//! - **Script Caching**: LRU cache for compiled scripts
//! - **Host Functions**: Exposed Deno ops for logging, HTTP fetch, and timers
//! - **Security**: Domain allowlist for fetch, private IP blocking, request limits

mod cache;
mod engine;
mod ops;
mod polyfills;
mod runtime;

pub use engine::ActionEngine;
