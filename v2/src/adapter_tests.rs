use super::*;
use crate::native_core::common::CorePaths;
use std::sync::{Arc, Mutex};
struct Recording {
    trace: Vec<(usize, usize)>,
    at: usize,
}
impl Choice for Recording {
    fn choose(&mut self, upper: usize) -> Result<usize, String> {
        let (bound, choice) = self.trace[self.at];
        self.at += 1;
        assert_eq!(upper, bound);
        Ok(choice)
    }
}
fn context(
    dir: &std::path::Path,
    answer: Result<Value, String>,
    record: Arc<Mutex<Vec<Vec<Value>>>>,
) -> CoreContext {
    let ctx = CoreContext::new(
        CorePaths {
            data_dir: dir.join("data"),
            config_dir: dir.join("config"),
            resource_dir: dir.join("resources"),
            temp_dir: dir.join("tmp"),
            home_dir: dir.into(),
        },
        None,
    );
    ctx.set_router(Arc::new(move |channel, args| {
        assert_eq!(channel, "clipboard:write");
        record.lock().unwrap().push(args);
        let answer = answer.clone();
        Box::pin(async move { answer })
    }))
    .unwrap();
    ctx
}
#[tokio::test]
async fn actual_tsx_commands_preferences_forms_and_copy_sequences() {
    let flows: Vec<Value> =
        serde_json::from_str(include_str!("../fixtures/oracle/commands.json")).unwrap();
    for flow in flows {
        let dir = tempfile::tempdir().unwrap();
        let record = Arc::new(Mutex::new(vec![]));
        let ctx = context(
            dir.path(),
            if flow["copyFailure"] == true {
                Err("private transport failure".into())
            } else {
                Ok(json!({"ok":true}))
            },
            record.clone(),
        );
        let mut rng = Recording {
            trace: serde_json::from_value(flow["trace"].clone()).unwrap(),
            at: 0,
        };
        let mut state = json!({});
        let command = flow["command"].as_str().unwrap();
        let prefs = flow["preferences"].clone();
        let mut result = handle_with(
            &ctx,
            command,
            "extension:run-command",
            &json!({"preferences":prefs}),
            &mut state,
            &mut rng,
        )
        .await
        .unwrap();
        if command != "copy-random-password" {
            let own = &flow["form"]["children"];
            let native = result["root"]["children"].as_array().unwrap();
            for field in own
                .as_array()
                .unwrap()
                .iter()
                .filter(|f| f.get("id").is_some())
            {
                let n = native
                    .iter()
                    .find(|n| n["props"]["id"] == field["id"])
                    .unwrap();
                assert_eq!(n["props"]["value"], field["defaultValue"]);
                assert!(n["props"].get("actionId").is_none()); // submit-only, no controlled echo
            }
            let action = state["generate"].clone();
            result = handle_with(
                &ctx,
                command,
                "extension:invoke-action",
                &json!({"preferences":prefs,"actionId":action,"formValues":flow["values"]}),
                &mut state,
                &mut rng,
            )
            .await
            .unwrap();
        }
        let expected: Vec<Vec<Value>> = flow["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e[0] == "copy")
            .map(|e| vec![e[1].clone()])
            .collect();
        assert_eq!(*record.lock().unwrap(), expected, "{command}");
        assert_eq!(rng.at, rng.trace.len());
        if !expected.is_empty() {
            let secret = expected[0][0].as_str().unwrap();
            assert!(
                !state.to_string().contains(secret),
                "state dump exposed password"
            );
            if flow["copyFailure"] == true {
                assert!(state.get("checkpoint").is_some());
                assert!(result["message"].as_str().unwrap().contains("Retry Copy"));
            } else {
                assert!(state.get("checkpoint").is_none());
            }
        }
    }
}
#[test]
fn strict_clipboard_ack_matrix() {
    for good in [
        json!({"ok":true}),
        json!({"ok":true,"error":null}),
        json!({"ok":true,"success":true,"error":null}),
    ] {
        assert!(ack(&good));
    }
    for bad in [
        Value::Null,
        json!(true),
        json!(false),
        json!({}),
        json!({"ok":true,"success":null}),
        json!({"ok":true,"success":false}),
        json!({"ok":true,"error":""}),
        json!({"ok":false}),
        json!({"ok":"true"}),
        json!([]),
    ] {
        assert!(!ack(&bad));
    }
}
#[tokio::test]
async fn cancellation_checkpoint_retry_same_secret_no_refresh_effects_or_stale_actions() {
    let dir = tempfile::tempdir().unwrap();
    let record = Arc::new(Mutex::new(vec![]));
    let ctx = context(dir.path(), Ok(json!({"ok":true})), record.clone());
    let mut state = json!({});
    let cmd = "generate-random-password";
    handle(&ctx, cmd, "extension:run-command", &json!({}), &mut state)
        .await
        .unwrap();
    let old_action = state["generate"].clone();
    let binding = state["binding"].clone();
    let blocked = ctx.clone(); // separate context with cancellable recording router
    let entered = Arc::new(tokio::sync::Notify::new());
    let entered2 = entered.clone();
    let block_ctx = CoreContext::new(blocked.paths.clone(), None);
    let rec = record.clone();
    block_ctx
        .set_router(Arc::new(move |channel, args| {
            assert_eq!(channel, "clipboard:write");
            rec.lock().unwrap().push(args);
            let entered = entered2.clone();
            Box::pin(async move {
                entered.notify_one();
                std::future::pending().await
            })
        }))
        .unwrap();
    let payload = json!({"actionId":old_action,"formValues":{"length":"20","useNumbers":true,"useSymbols":true}});
    {
        let future = handle(
            &block_ctx,
            cmd,
            "extension:invoke-action",
            &payload,
            &mut state,
        );
        tokio::pin!(future);
        tokio::select! { _=entered.notified()=>{},result=&mut future=>panic!("unexpected {result:?}") }
    }
    assert_eq!(state["binding"], binding);
    let checkpoint = state.clone();
    struct NoEntropy;
    impl Choice for NoEntropy {
        fn choose(&mut self, _: usize) -> Result<usize, String> {
            panic!("heartbeat drew entropy")
        }
    }
    for operation in [
        "extension:refresh-session",
        "extension:search-text-changed",
        "extension:load-more",
    ] {
        let mut staged = state.clone();
        let response = handle_with(
            &ctx,
            cmd,
            operation,
            &json!({}),
            &mut staged,
            &mut NoEntropy,
        )
        .await
        .unwrap();
        assert_eq!(response["mode"], "view");
        assert_eq!(response["actions"][0]["id"], checkpoint["retry"]);
        assert_eq!(staged, checkpoint);
    }
    assert!(handle(
        &ctx,
        cmd,
        "extension:invoke-action",
        &json!({"actionId":old_action,"formValues":payload["formValues"]}),
        &mut state
    )
    .await
    .is_err());
    let mut other = json!({});
    handle(&ctx, cmd, "extension:run-command", &json!({}), &mut other)
        .await
        .unwrap();
    assert!(handle(
        &ctx,
        cmd,
        "extension:invoke-action",
        &json!({"actionId":other["generate"],"formValues":payload["formValues"]}),
        &mut state
    )
    .await
    .is_err());
    let retry = state["retry"].clone();
    handle_with(
        &ctx,
        cmd,
        "extension:invoke-action",
        &json!({"actionId":retry}),
        &mut state,
        &mut NoEntropy,
    )
    .await
    .unwrap();
    {
        let copies = record.lock().unwrap();
        assert_eq!(copies.len(), 2);
        assert_eq!(copies[0], copies[1]);
    }
    assert!(state.get("checkpoint").is_none());
    assert!(handle(
        &ctx,
        cmd,
        "extension:invoke-action",
        &json!({"actionId":retry}),
        &mut state
    )
    .await
    .is_err());
    handle(
        &ctx,
        cmd,
        "extension:dispose-session",
        &json!({}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(state, json!({}));
}
#[tokio::test]
async fn settings_preferences_unknown_keys_concurrency_and_mid_session_corruption() {
    let dir = tempfile::tempdir().unwrap();
    let record = Arc::new(Mutex::new(vec![]));
    let ctx = context(dir.path(), Ok(json!({"ok":true})), record.clone());
    let path = ctx
        .paths
        .data_dir
        .join("native-extensions-v2/password-generator/storage.json");
    sdk::update_store(&ctx,SLUG,|v|{*v=json!({"unknown":{"keep":true},"length":"32","useNumbers":false,"useSymbols":false,"format":"hello-{number:3}"});Ok(())}).await.unwrap();
    for disabled in [false, true] {
        let mut state = json!({});
        let prefs = json!({"storePasswordLength":!disabled,"rememberCharacterGroups":!disabled,"rememberCustomFormat":!disabled});
        handle(
            &ctx,
            "generate-random-password",
            "extension:run-command",
            &json!({"preferences":prefs}),
            &mut state,
        )
        .await
        .unwrap();
        assert_eq!(
            state["defaults"]["length"],
            if disabled { "20" } else { "32" }
        );
        assert_eq!(state["defaults"]["useNumbers"], disabled);
        let action = state["generate"].clone();
        let before = std::fs::read(&path).unwrap();
        handle(&ctx,"generate-random-password","extension:invoke-action",&json!({"actionId":action,"formValues":{"length":"32","useNumbers":false,"useSymbols":false}}),&mut state).await.unwrap();
        if disabled {
            assert_eq!(std::fs::read(&path).unwrap(), before);
        }
        assert_eq!(
            sdk::read_store(&ctx, SLUG).await.unwrap()["unknown"],
            json!({"keep":true})
        );
    }
    let mut tasks = vec![];
    for _ in 0..25 {
        let ctx = ctx.clone();
        tasks.push(tokio::spawn(async move {
            sdk::update_store(&ctx, SLUG, |v| {
                v["counter"] = json!(v["counter"].as_u64().unwrap_or(0) + 1);
                Ok(())
            })
            .await
            .unwrap()
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }
    let store = sdk::read_store(&ctx, SLUG).await.unwrap();
    assert_eq!(store["counter"], 25);
    assert_eq!(store["unknown"], json!({"keep":true}));
    for corrupt in [
        "{broken",
        "[]",
        "{\"length\":10}",
        "{\"useNumbers\":\"true\"}",
        "{\"format\":false}",
    ] {
        std::fs::write(&path, serde_json::to_vec(&store).unwrap()).unwrap();
        let mut state = json!({});
        handle(
            &ctx,
            "generate-random-password",
            "extension:run-command",
            &json!({}),
            &mut state,
        )
        .await
        .unwrap();
        std::fs::write(&path, corrupt).unwrap();
        let count = record.lock().unwrap().len();
        let action = state["generate"].clone();
        let result=handle(&ctx,"generate-random-password","extension:invoke-action",&json!({"actionId":action,"formValues":{"length":"20","useNumbers":true,"useSymbols":true}}),&mut state).await.unwrap();
        assert_eq!(result["message"], SETTINGS_ERROR);
        assert_eq!(std::fs::read(&path).unwrap(), corrupt.as_bytes());
        assert_eq!(record.lock().unwrap().len(), count);
        let mut next = json!({});
        assert!(handle(
            &ctx,
            "generate-random-password",
            "extension:run-command",
            &json!({}),
            &mut next
        )
        .await
        .is_err());
    }
}
#[tokio::test]
async fn routed_ack_failures_entropy_errors_and_canonical_copy_binding() {
    for answer in [
        Ok(Value::Null),
        Ok(json!(true)),
        Ok(json!({"ok":true,"success":false})),
        Ok(json!({"ok":true,"error":"private secret service error"})),
        Err("private service failure".into()),
        Ok(json!({"ok":true})),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let record = Arc::new(Mutex::new(vec![]));
        let success = answer.as_ref().is_ok_and(ack);
        let ctx = context(dir.path(), answer, record.clone());
        let mut state = json!({});
        let result = handle(
            &ctx,
            "copy-random-password",
            "extension:run-command",
            &json!({"preferences":{"length":"5"}}),
            &mut state,
        )
        .await
        .unwrap();
        assert_eq!(record.lock().unwrap().len(), 1);
        assert!(!result.to_string().contains("private"));
        assert_eq!(state.get("checkpoint").is_none(), success);
        assert_eq!(result["mode"], if success { "no-view" } else { "view" });
        if !success {
            let retry = state["retry"].clone();
            handle(&ctx,"copy-random-password","extension:invoke-action",&json!({"actionId":retry,"text":"forged output","password":"forged output","formValues":{"length":"64"}}),&mut state).await.unwrap();
            let record = record.lock().unwrap();
            assert_eq!(record.len(), 2);
            assert_eq!(record[0], record[1]);
        }
    }
    struct Fail;
    impl Choice for Fail {
        fn choose(&mut self, _: usize) -> Result<usize, String> {
            Err(engine::ENTROPY_ERROR.into())
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let record = Arc::new(Mutex::new(vec![]));
    let ctx = context(dir.path(), Ok(json!({"ok":true})), record.clone());
    let mut state = json!({});
    handle_with(
        &ctx,
        "copy-random-password",
        "extension:run-command",
        &json!({}),
        &mut state,
        &mut Fail,
    )
    .await
    .unwrap();
    assert!(record.lock().unwrap().is_empty());
    assert!(state.get("checkpoint").is_none());
    let mut a = json!({"binding":"session-a"});
    checkpoint(&mut a, "public-test-fixture").unwrap();
    let mut b = a.clone();
    b["binding"] = json!("session-b");
    assert!(unseal(&b).is_err());
    let bytes = a["checkpoint"]["ciphertext"].as_array_mut().unwrap();
    bytes[0] = json!(bytes[0].as_u64().unwrap() ^ 1);
    assert!(unseal(&a).is_err());
}
