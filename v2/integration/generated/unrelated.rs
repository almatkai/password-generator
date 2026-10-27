pub mod v2_amphetamine {
 pub fn commands() -> &'static [&'static str] { &["amp-start", "quick-start", "amp-end", "remaining", "amp-duration"] }
 pub async fn handle(_: &crate::native_core::common::CoreContext,_: &str,_: &str,_: &serde_json::Value,_: &mut serde_json::Value)->crate::native_core::common::CoreResult { panic!("unrelated adapter seam: amphetamine") }
}
pub mod v2_audio_device {
 pub fn commands() -> &'static [&'static str] { &["set-output-device", "set-input-device", "set-volume", "set-input-volume", "auto-switch-input", "auto-switch-output", "customize-order", "favourite", "toggle-favourites", "use-combo1", "use-combo2", "use-combo3"] }
 pub async fn handle(_: &crate::native_core::common::CoreContext,_: &str,_: &str,_: &serde_json::Value,_: &mut serde_json::Value)->crate::native_core::common::CoreResult { panic!("unrelated adapter seam: audio-device") }
}
pub mod v2_cleanshotx {
 pub fn commands() -> &'static [&'static str] { &["open-history", "open-settings", "view-uploads", "all-in-one", "capture-fullscreen", "capture-area", "capture-previous-area", "capture-window", "self-timer", "scrolling-capture", "record-screen", "capture-text", "annotate", "open-from-clipboard", "toggle-desktop-icons", "hide-desktop-icons", "show-desktop-icons", "restore-recently-closed", "add-quick-access-overlay", "pin", "capture-area-raycast-aichat", "record-screen-preset", "manage-recording-presets"] }
 pub async fn handle(_: &crate::native_core::common::CoreContext,_: &str,_: &str,_: &serde_json::Value,_: &mut serde_json::Value)->crate::native_core::common::CoreResult { panic!("unrelated adapter seam: cleanshotx") }
}
pub mod v2_color_picker {
 pub fn commands() -> &'static [&'static str] { &["pick-color", "extract-color", "menu-bar", "organize-colors", "favorite-colors", "generate-colors", "color-names", "color-wheel", "convert-color"] }
 pub async fn handle(_: &crate::native_core::common::CoreContext,_: &str,_: &str,_: &serde_json::Value,_: &mut serde_json::Value)->crate::native_core::common::CoreResult { panic!("unrelated adapter seam: color-picker") }
}
pub mod v2_gif_search {
 pub fn commands() -> &'static [&'static str] { &["search", "favorites", "recents"] }
 pub async fn handle(_: &crate::native_core::common::CoreContext,_: &str,_: &str,_: &serde_json::Value,_: &mut serde_json::Value)->crate::native_core::common::CoreResult { panic!("unrelated adapter seam: gif-search") }
}
pub mod v2_google_calendar {
 pub fn commands() -> &'static [&'static str] { &["create-event", "create-quick-event", "search-contacts", "list-events", "list-calendars"] }
 pub async fn handle(_: &crate::native_core::common::CoreContext,_: &str,_: &str,_: &serde_json::Value,_: &mut serde_json::Value)->crate::native_core::common::CoreResult { panic!("unrelated adapter seam: google-calendar") }
}
pub mod v2_google_search {
 pub fn commands() -> &'static [&'static str] { &["index", "searchSelectText"] }
 pub async fn handle(_: &crate::native_core::common::CoreContext,_: &str,_: &str,_: &serde_json::Value,_: &mut serde_json::Value)->crate::native_core::common::CoreResult { panic!("unrelated adapter seam: google-search") }
}
pub mod v2_perplexity {
 pub fn commands() -> &'static [&'static str] { &["ask-perplexity"] }
 pub async fn handle(_: &crate::native_core::common::CoreContext,_: &str,_: &str,_: &serde_json::Value,_: &mut serde_json::Value)->crate::native_core::common::CoreResult { panic!("unrelated adapter seam: perplexity") }
}
pub mod v2_port_manager {
 pub fn commands() -> &'static [&'static str] { &["kill-listening-process", "named-ports", "open-ports", "open-ports-menu-bar"] }
 pub async fn handle(_: &crate::native_core::common::CoreContext,_: &str,_: &str,_: &serde_json::Value,_: &mut serde_json::Value)->crate::native_core::common::CoreResult { panic!("unrelated adapter seam: port-manager") }
}
pub mod v2_speedtest {
 pub fn commands() -> &'static [&'static str] { &["index"] }
 pub async fn handle(_: &crate::native_core::common::CoreContext,_: &str,_: &str,_: &serde_json::Value,_: &mut serde_json::Value)->crate::native_core::common::CoreResult { panic!("unrelated adapter seam: speedtest") }
}
pub mod v2_timers {
 pub fn commands() -> &'static [&'static str] { &["start2MinuteTimer", "start5MinuteTimer", "start10MinuteTimer", "start15MinuteTimer", "start25MinuteTimer", "start30MinuteTimer", "start45MinuteTimer", "start60MinuteTimer", "start90MinuteTimer", "startCustomTimer", "manageTimers", "timersMenuBar", "startStopwatch", "manageStopwatches", "dismissTimerAlert", "stopwatchMenuBar", "previewAlertSounds", "stopRunningTimer", "configureMenubarPresets"] }
 pub async fn handle(_: &crate::native_core::common::CoreContext,_: &str,_: &str,_: &serde_json::Value,_: &mut serde_json::Value)->crate::native_core::common::CoreResult { panic!("unrelated adapter seam: timers") }
}
