//! Compile-only host seam: checks non-test adapter code against real getrandom.
//! Never executes the adapter or OS operations. Tauri events remain a seam.
extern crate self as tauri;
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
#[path = "../tests/harness/native_core.rs"]
pub mod native_core;
fn main() {}
