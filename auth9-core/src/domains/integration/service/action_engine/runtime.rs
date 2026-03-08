//! Thread-local V8 runtime management.
//!
//! JsRuntime is !Send and must stay on one thread.
//! This module provides thread-local storage and helper functions
//! for creating, taking, and returning JsRuntime instances.

use crate::domain::action::AsyncActionConfig;
use crate::error::{AppError, Result};
use deno_core::{JsRuntime, RuntimeOptions};
use std::cell::RefCell;

use super::ops::{auth9_action_ext, RequestCounter};
use super::polyfills::POLYFILLS_JS;

// Thread-local storage for V8 runtime (JsRuntime is !Send, must stay on one thread)
thread_local! {
    static JS_RUNTIME: RefCell<Option<JsRuntime>> = const { RefCell::new(None) };
    static LOCAL_TOKIO_RT: RefCell<Option<tokio::runtime::Runtime>> = const { RefCell::new(None) };
}

/// Take JsRuntime out of thread-local (avoids RefCell borrow across await)
pub(super) fn take_js_runtime() -> Option<JsRuntime> {
    JS_RUNTIME.with(|rt| rt.borrow_mut().take())
}

/// Return JsRuntime to thread-local storage
pub(super) fn return_js_runtime(runtime: JsRuntime) {
    JS_RUNTIME.with(|rt| {
        *rt.borrow_mut() = Some(runtime);
    });
}

/// Create a new JsRuntime with async extension + polyfills
pub(super) fn create_js_runtime(
    http_client: reqwest::Client,
    config: AsyncActionConfig,
) -> Result<JsRuntime> {
    tracing::debug!("Creating thread-local V8 runtime with async extensions");

    let max_heap_bytes = config.max_heap_mb * 1024 * 1024;
    let create_params = deno_core::v8::Isolate::create_params().heap_limits(0, max_heap_bytes);

    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![auth9_action_ext::init_ops_and_esm()],
        create_params: Some(create_params),
        ..Default::default()
    });

    // Inject initial op state
    {
        let op_state = runtime.op_state();
        let mut state = op_state.borrow_mut();
        state.put(http_client);
        state.put(config);
        state.put(RequestCounter(0));
    }

    // Inject polyfills (fetch, setTimeout, console)
    runtime
        .execute_script("<polyfills>", POLYFILLS_JS)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to load polyfills: {}", e)))?;

    Ok(runtime)
}

/// Get or create thread-local tokio current-thread runtime, take it out for use
pub(super) fn take_local_tokio_rt() -> tokio::runtime::Runtime {
    LOCAL_TOKIO_RT.with(|rt_cell| {
        let mut rt = rt_cell.borrow_mut();
        if rt.is_none() {
            *rt = Some(
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create thread-local tokio runtime"),
            );
        }
        rt.take().unwrap()
    })
}

/// Return tokio runtime to thread-local storage
pub(super) fn return_local_tokio_rt(rt: tokio::runtime::Runtime) {
    LOCAL_TOKIO_RT.with(|rt_cell| {
        *rt_cell.borrow_mut() = Some(rt);
    });
}
