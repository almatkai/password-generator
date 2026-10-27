// Real beta functions; Tauri emission is a test-only seam.
extern crate self as tauri;
// Test target dependency seam only. Host and library builds link the real locked
// getrandom crate. These public fixture bytes never become production defaults.
extern crate self as getrandom;
pub fn getrandom(bytes: &mut [u8]) -> Result<(), std::convert::Infallible> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static PUBLIC_FIXTURE_COUNTER: AtomicU32 = AtomicU32::new(1);
    for chunk in bytes.chunks_mut(4) {
        let value = PUBLIC_FIXTURE_COUNTER
            .fetch_add(17, Ordering::Relaxed)
            .to_le_bytes();
        chunk.copy_from_slice(&value[..chunk.len()]);
    }
    Ok(())
}
#[derive(Clone)]
pub struct AppHandle;
pub trait Emitter {
    fn emit(&self, channel: &str, payload: serde_json::Value) -> Result<(), String>;
}
impl Emitter for AppHandle {
    fn emit(&self, _: &str, _: serde_json::Value) -> Result<(), String> {
        panic!("Tauri OS seam must not be called")
    }
}
#[path = "harness/native_core.rs"]
pub mod native_core;
