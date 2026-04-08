//! WASM bindings land after the native contracts are stable.
//! The thin slice keeps the crate compiling so the workspace shape is already fixed.

pub fn wasm_ready() -> bool {
    false
}
