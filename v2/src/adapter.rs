//! Compiled as `extensions::v2_password_generator`, sibling of the actual beta SDK.
use super::v2_sdk as sdk;
use crate::native_core::common::{CoreContext, CoreResult};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    OnceLock,
};
use zeroize::Zeroizing;
#[path = "engine.rs"]
mod engine;
use engine::{Choice, OsChoice};
const SLUG: &str = "password-generator";
const COMMANDS: &[&str] = &[
    "generate-random-password",
    "copy-random-password",
    "generate-memorable-password",
];
const SETTINGS_ERROR: &str =
    "Could not read or save password settings. Repair the settings file before generating again.";
const COPY_ERROR: &str = "Copy could not be confirmed. Retry Copy uses the same password.";
static PROCESS: OnceLock<Result<[u8; 52], ()>> = OnceLock::new();
static SEQUENCE: AtomicU64 = AtomicU64::new(1);
fn process() -> Result<&'static [u8; 52], String> {
    PROCESS
        .get_or_init(|| {
            let mut bytes = [0; 52];
            getrandom::getrandom(&mut bytes).map_err(|_| ())?;
            Ok(bytes)
        })
        .as_ref()
        .map_err(|_| engine::ENTROPY_ERROR.into())
}
fn sequence() -> Result<u64, String> {
    SEQUENCE
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .map_err(|_| engine::ENTROPY_ERROR.into())
}
fn capability() -> Result<String, String> {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut prefix = String::with_capacity(32);
    for b in &process()?[32..48] {
        prefix.push(HEX[(b >> 4) as usize] as char);
        prefix.push(HEX[(b & 15) as usize] as char);
    }
    Ok(format!("password-generator:{prefix}:{}", sequence()?))
}
fn seal(secret: &str, binding: &str) -> Result<Value, String> {
    let process = process()?;
    let cipher =
        ChaCha20Poly1305::new_from_slice(&process[..32]).map_err(|_| engine::ENTROPY_ERROR)?;
    let mut nonce = [0; 12];
    nonce[..4].copy_from_slice(&process[48..52]);
    nonce[4..].copy_from_slice(&sequence()?.to_be_bytes());
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: secret.as_bytes(),
                aad: binding.as_bytes(),
            },
        )
        .map_err(|_| engine::ENTROPY_ERROR)?;
    Ok(json!({"nonce":nonce.to_vec(),"ciphertext":ciphertext}))
}
fn unseal(state: &Value) -> Result<Zeroizing<String>, String> {
    let binding = state["binding"]
        .as_str()
        .ok_or("Password checkpoint unavailable. Generate again.")?;
    let nonce: Vec<u8> = serde_json::from_value(state["checkpoint"]["nonce"].clone())
        .map_err(|_| "Password checkpoint unavailable. Generate again.")?;
    let ciphertext: Vec<u8> = serde_json::from_value(state["checkpoint"]["ciphertext"].clone())
        .map_err(|_| "Password checkpoint unavailable. Generate again.")?;
    if nonce.len() != 12 {
        return Err("Password checkpoint unavailable. Generate again.".into());
    }
    let cipher =
        ChaCha20Poly1305::new_from_slice(&process()?[..32]).map_err(|_| engine::ENTROPY_ERROR)?;
    let bytes = Zeroizing::new(
        cipher
            .decrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: &ciphertext,
                    aad: binding.as_bytes(),
                },
            )
            .map_err(|_| "Password checkpoint unavailable. Generate again.")?,
    );
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| "Password checkpoint unavailable. Generate again.")?;
    Ok(Zeroizing::new(text.to_owned()))
}
pub fn commands() -> &'static [&'static str] {
    COMMANDS
}
fn boolean(prefs: &Value, key: &str, default: bool) -> Result<bool, String> {
    match prefs.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Bool(v)) => Ok(*v),
        _ => Err("Invalid password preferences.".into()),
    }
}
fn string(prefs: &Value, key: &str, default: &str) -> Result<String, String> {
    match prefs.get(key) {
        None | Some(Value::Null) => Ok(default.into()),
        Some(Value::String(v)) => Ok(v.clone()),
        _ => Err("Invalid password preferences.".into()),
    }
}
fn validate_store(v: &Value) -> Result<(), String> {
    if !v.is_object() {
        return Err(SETTINGS_ERROR.into());
    }
    for k in ["length", "format"] {
        if v.get(k).is_some_and(|v| !v.is_string()) {
            return Err(SETTINGS_ERROR.into());
        }
    }
    for k in ["useNumbers", "useSymbols"] {
        if v.get(k).is_some_and(|v| !v.is_boolean()) {
            return Err(SETTINGS_ERROR.into());
        }
    }
    Ok(())
}
fn field(id: &str, kind: &str, title: &str, value: Value) -> Value {
    json!({"type":kind,"props":{"id":id,"title":title,"label":title,"value":value}})
}
fn view(command: &str, state: &Value, message: Option<&str>) -> Value {
    let mut fields = Vec::new();
    let title = if command == "generate-memorable-password" {
        "Memorable Password"
    } else {
        "Password Generator"
    };
    if command == "generate-random-password" {
        fields.push(field(
            "length",
            "Form.TextField",
            "Number of Characters",
            state["defaults"]["length"].clone(),
        ));
        fields.push(field(
            "useNumbers",
            "Form.Checkbox",
            "Use numbers",
            state["defaults"]["useNumbers"].clone(),
        ));
        fields.push(field(
            "useSymbols",
            "Form.Checkbox",
            "Use symbols",
            state["defaults"]["useSymbols"].clone(),
        ));
        fields.push(json!({"type":"Form.Description","props":{"text":"Ambiguous characters such as 0, O, 1, and l are excluded."}}));
    } else if command == "generate-memorable-password" {
        fields.push(field(
            "format",
            "Form.TextField",
            "Format",
            state["defaults"]["format"].clone(),
        ));
        fields.push(json!({"type":"Form.Description","props":{"text":"Tokens: {word}, {word:uppercase}, {word:lowercase}, {word:capitalize}, {number:n}, {symbol:n}, and {random:n}."}}));
    }
    let mut actions = vec![
        json!({"id":state["generate"],"title":"Generate and Copy Password","kind":"submit-form"}),
    ];
    if state.get("checkpoint").is_some() {
        actions.insert(
            0,
            json!({"id":state["retry"],"title":"Retry Copy","kind":"action"}),
        );
    }
    let mut out = sdk::view(
        title,
        json!({"type":"Form","props":{"navigationTitle":title},"children":fields}),
        actions,
    );
    if let Some(message) = message {
        out["message"] = json!(message);
    }
    out
}
fn ack(value: &Value) -> bool {
    value.is_object()
        && value["ok"] == true
        && value.get("error").is_none_or(Value::is_null)
        && value.get("success").is_none_or(|v| v == true)
}
async fn copy_checkpoint(ctx: &CoreContext, command: &str, state: &mut Value) -> CoreResult {
    let secret = unseal(state)?;
    // Intent/checkpoint already committed synchronously. A dropped future must
    // leave retry available, without generating, saving, or copying on refresh.
    state["intent"] = json!("copy-uncertain");
    let result = sdk::copy(ctx, &secret).await;
    if !result.as_ref().is_ok_and(ack) {
        return Ok(view(command, state, Some(COPY_ERROR)));
    }
    state
        .as_object_mut()
        .ok_or("Invalid password session")?
        .remove("checkpoint");
    state.as_object_mut().unwrap().remove("retry");
    state["intent"] = json!("copied");
    let title = if command == "generate-memorable-password" {
        "Memorable password copied"
    } else {
        "Password copied"
    };
    if command == "copy-random-password" {
        return Ok(sdk::no_view(title));
    }
    // Supported beta adaptation: no invented HUD, hide, popToRoot, or NULL ack.
    // The source's hideAfterCopy preference means return to root; beta lacks it.
    let message = if state["hideAfterCopy"] == true {
        title.to_owned()
    } else {
        format!("{title}: {}", *secret)
    };
    Ok(view(command, state, Some(&message)))
}
pub async fn handle(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    handle_with(ctx, command, operation, payload, state, &mut OsChoice).await
}
async fn handle_with(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
    rng: &mut impl Choice,
) -> CoreResult {
    if !COMMANDS.contains(&command) {
        return Err("Unknown password command.".into());
    }
    match operation {
        "extension:dispose-session" => {
            *state = json!({});
            return Ok(sdk::no_view("Password session closed"));
        }
        "extension:refresh-session" | "extension:search-text-changed" | "extension:load-more" => {
            return Ok(if state.get("checkpoint").is_some() {
                view(command, state, Some(COPY_ERROR))
            } else {
                sdk::unchanged()
            });
        }
        "extension:run-command" => {
            if state.get("binding").is_some() {
                return Err("Password session already launched.".into());
            }
            let prefs = &payload["preferences"];
            let hide = boolean(prefs, "hideAfterCopy", false)?;
            let mut defaults = json!({"length":"20","useNumbers":true,"useSymbols":true,"format":engine::DEFAULT_FORMAT});
            let remember_length = boolean(prefs, "storePasswordLength", true)?;
            let remember_groups = boolean(prefs, "rememberCharacterGroups", true)?;
            let remember_format = boolean(prefs, "rememberCustomFormat", true)?;
            if command != "copy-random-password" {
                let store = sdk::read_store(ctx, SLUG)
                    .await
                    .map_err(|_| SETTINGS_ERROR)?;
                validate_store(&store)?;
                for (key, enabled) in [
                    ("length", remember_length),
                    ("useNumbers", remember_groups),
                    ("useSymbols", remember_groups),
                    ("format", remember_format),
                ] {
                    if enabled {
                        if let Some(value) = store.get(key) {
                            defaults[key] = value.clone();
                        }
                    }
                }
            }
            *state = json!({"binding":format!("{}:{command}",capability()?),"command":command,"generate":capability()?,"defaults":defaults,"hideAfterCopy":hide,
              "rememberLength":remember_length,"rememberGroups":remember_groups,"rememberFormat":remember_format});
            if command != "copy-random-password" {
                return Ok(view(command, state, None));
            }
            // No-view command copies immediately at launch. No confirmation step.
            let length = string(prefs, "length", "20")?;
            let numbers = boolean(prefs, "useNumbers", true)?;
            let symbols = boolean(prefs, "useSymbols", true)?;
            let secret=match engine::parse_length(&length).and_then(|n|engine::random_with(n,numbers,symbols,rng)) {
                Ok(secret)=>secret,Err(_)=>return Ok(view(command,state,Some("Could not generate password. Check length (5 to 64) and preferences, then Generate again.")))
            };
            checkpoint(state, &secret)?;
            drop(secret);
            return copy_checkpoint(ctx, command, state).await;
        }
        "extension:invoke-action" => {}
        _ => return Err("Unsupported password operation.".into()),
    }
    if state["command"] != command {
        return Err("Password session unavailable.".into());
    }
    let action = payload["actionId"]
        .as_str()
        .ok_or("Missing password action.")?;
    if state.get("retry").and_then(Value::as_str) == Some(action)
        && state.get("checkpoint").is_some()
    {
        return copy_checkpoint(ctx, command, state).await;
    }
    if state["generate"].as_str() != Some(action) {
        return Err("Stale or invalid password action. Use the current form.".into());
    }
    let values = &payload["formValues"];
    let mut settings = json!({});
    let result = if command == "generate-memorable-password" {
        let raw = values["format"]
            .as_str()
            .ok_or("Submit the complete password form.")?;
        let format = engine::js_trim(raw);
        let format = if format.is_empty() {
            engine::DEFAULT_FORMAT
        } else {
            format
        };
        if state["rememberFormat"] == true {
            settings["format"] = json!(raw);
        }
        engine::memorable_with(format, rng)
    } else {
        let (length, numbers, symbols) = if command == "copy-random-password" {
            let prefs = &payload["preferences"];
            (
                string(prefs, "length", "20")?,
                boolean(prefs, "useNumbers", true)?,
                boolean(prefs, "useSymbols", true)?,
            )
        } else {
            (
                values["length"]
                    .as_str()
                    .ok_or("Submit the complete password form.")?
                    .to_owned(),
                values["useNumbers"]
                    .as_bool()
                    .ok_or("Submit the complete password form.")?,
                values["useSymbols"]
                    .as_bool()
                    .ok_or("Submit the complete password form.")?,
            )
        };
        if command == "generate-random-password" {
            if state["rememberLength"] == true {
                settings["length"] = json!(length);
            }
            if state["rememberGroups"] == true {
                settings["useNumbers"] = json!(numbers);
                settings["useSymbols"] = json!(symbols);
            }
        }
        engine::parse_length(&length).and_then(|n| engine::random_with(n, numbers, symbols, rng))
    };
    let secret = match result {
        Ok(v) => v,
        Err(e) => return Ok(view(command, state, Some(&e))),
    };
    checkpoint(state, &secret)?;
    drop(secret);
    // Revalidate inside the SDK's process-atomic transaction, including changes
    // made by another session since launch. No generated password enters storage.
    if !settings.as_object().is_none_or(|m| m.is_empty()) {
        let result = sdk::update_store(ctx, SLUG, move |store| {
            validate_store(store)?;
            let target = store.as_object_mut().ok_or(SETTINGS_ERROR)?;
            for (k, v) in settings.as_object().ok_or(SETTINGS_ERROR)? {
                target.insert(k.clone(), v.clone());
            }
            Ok(())
        })
        .await;
        if result.is_err() {
            return Ok(view(command, state, Some(SETTINGS_ERROR)));
        }
    }
    copy_checkpoint(ctx, command, state).await
}
fn checkpoint(state: &mut Value, secret: &str) -> Result<(), String> {
    let binding = state["binding"]
        .as_str()
        .ok_or("Invalid password session")?;
    let sealed = seal(secret, binding)?;
    let generate = capability()?;
    let retry = capability()?;
    state["checkpoint"] = sealed;
    state["generate"] = json!(generate);
    state["retry"] = json!(retry);
    state["intent"] = json!("copy-uncertain");
    Ok(())
}
#[cfg(test)]
#[path = "adapter_tests.rs"]
mod tests;
