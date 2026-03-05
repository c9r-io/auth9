//! JavaScript polyfills injected once at V8 runtime creation.

/// Polyfills for fetch, setTimeout, clearTimeout, and console (injected into globalThis).
pub(super) const POLYFILLS_JS: &str = r#"
// fetch(url, options?) -> Promise<{ status, body, headers, ok, text(), json() }>
globalThis.fetch = async function(url, options) {
    options = options || {};
    const method = (options.method || 'GET').toUpperCase();
    const headers = options.headers || {};
    const body = options.body || '';
    const result = await Deno.core.ops.op_fetch(url, method, headers, body);
    return {
        status: result.status,
        ok: result.status >= 200 && result.status < 300,
        headers: result.headers,
        text: async () => result.body,
        json: async () => JSON.parse(result.body),
    };
};

// setTimeout(callback, delay) -> id
globalThis.__timers = { nextId: 1, pending: new Map() };
globalThis.setTimeout = function(callback, delay) {
    delay = delay || 0;
    const id = globalThis.__timers.nextId++;
    const promise = Deno.core.ops.op_set_timeout(delay).then(() => {
        if (globalThis.__timers.pending.has(id)) {
            globalThis.__timers.pending.delete(id);
            callback();
        }
    });
    globalThis.__timers.pending.set(id, promise);
    return id;
};
globalThis.clearTimeout = function(id) {
    globalThis.__timers.pending.delete(id);
};

// console.log/warn/error
globalThis.console = {
    log: (...args) => Deno.core.ops.op_console_log(args.map(String)),
    warn: (...args) => Deno.core.ops.op_console_log(args.map(a => '[WARN] ' + String(a))),
    error: (...args) => Deno.core.ops.op_console_log(args.map(a => '[ERROR] ' + String(a))),
};
"#;
