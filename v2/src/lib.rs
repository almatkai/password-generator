//! Offline password engine. The host compiles adapter.rs as a sibling of v2_sdk.
pub mod engine;
pub fn generate_random(
    length: usize,
    numbers: bool,
    symbols: bool,
) -> Result<zeroize::Zeroizing<String>, String> {
    engine::random_with(length, numbers, symbols, &mut engine::OsChoice)
}
pub fn generate_memorable(format: &str) -> Result<zeroize::Zeroizing<String>, String> {
    engine::memorable_with(format, &mut engine::OsChoice)
}
