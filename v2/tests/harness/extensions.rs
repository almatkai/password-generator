#[allow(dead_code)]
#[path = "../../integration/generated/harness_native.rs"]
pub mod native;
#[allow(dead_code)]
#[path = "../../integration/generated/harness_registry.rs"]
pub mod registry;
#[allow(dead_code)]
#[path = "../../integration/generated/harness_v2_catalog.rs"]
pub mod v2_catalog;
#[path = "../../src/adapter.rs"]
pub mod v2_password_generator;
#[allow(dead_code)]
#[path = "../../integration/generated/harness_v2_runtime.rs"]
pub mod v2_runtime;
#[allow(dead_code)]
#[path = "../../integration/generated/harness_v2_sdk.rs"]
pub mod v2_sdk;
include!("../../integration/generated/unrelated.rs");
#[allow(dead_code)]
#[path = "../../integration/generated/harness_current_registry.rs"]
mod current_registry;
#[cfg(test)]
#[path = "session_tests.rs"]
mod session_tests;
