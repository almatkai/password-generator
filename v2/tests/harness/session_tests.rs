use super::*;
use crate::native_core::common::{write_json_atomic, CoreContext, CorePaths};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
fn context(dir: &std::path::Path) -> CoreContext {
    CoreContext::new(
        CorePaths {
            data_dir: dir.join("data"),
            config_dir: dir.join("config"),
            resource_dir: dir.join("resources"),
            temp_dir: dir.join("tmp"),
            home_dir: dir.into(),
        },
        None,
    )
}
fn install(ctx: &CoreContext) {
    v2_catalog::install(ctx, "com.tezbar.native.password-generator")
        .unwrap()
        .unwrap();
}
#[tokio::test]
async fn real_registry_catalog_native_sessions_aliases_preferences_and_stale_actions() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = context(dir.path());
    install(&ctx);
    assert!(v2_runtime::supports(
        "raycast.password-generator",
        "copy-random-password"
    ));
    assert!(v2_runtime::supports(
        "com.tezbar.native.password-generator",
        "generate-memorable-password"
    ));
    assert!(!v2_runtime::supports(
        "raycast.password-generator",
        "forged"
    ));
    assert_eq!(v2_catalog::manifests().len(), 12);
    let manifest = v2_catalog::manifest("password-generator").unwrap();
    assert_eq!(manifest["status"], "complete");
    assert_eq!(manifest["totalCommands"], 3);
    assert!(manifest["verification"]
        .as_array()
        .unwrap()
        .iter()
        .all(|v| v["state"] == "not-run"));
    // Existing legacy package/preferences remain in place; native commands win aliases.
    let legacy = ctx.paths.data_dir.join("extensions/password-generator");
    write_json_atomic(&legacy.join("package.json"),&json!({"name":"password-generator","commands":[{"name":"copy-random-password","platforms":["macOS","Windows"]}],"platforms":["macOS","Windows"]})).unwrap();
    write_json_atomic(&legacy.join("preferences.json"),&json!({"commands":{"copy-random-password":{"length":"5","useNumbers":false,"useSymbols":false}}})).unwrap();
    let calls = Arc::new(Mutex::new(vec![]));
    let record = calls.clone();
    ctx.set_router(Arc::new(move |channel, args| {
        assert_eq!(channel, "clipboard:write");
        record.lock().unwrap().push(args);
        Box::pin(async { Ok(json!({"ok":true})) })
    }))
    .unwrap();
    let sessions = native::NativeSessions::default();
    let files = Arc::new(Mutex::new(()));
    let no_view=sessions.handle(&ctx,&files,"extension:run-command",&[json!({"extensionId":"raycast.password-generator","commandName":"copy-random-password","argumentValues":{"ignored":"anything"}})]).await.unwrap().unwrap();
    assert_eq!(no_view["mode"], "no-view");
    assert!(no_view.get("sessionId").is_none());
    assert_eq!(calls.lock().unwrap()[0][0].as_str().unwrap().len(), 5);
    assert!(legacy.join("package.json").exists());
    let launch = json!({"extensionId":"com.tezbar.native.password-generator","commandName":"generate-random-password"});
    let form = sessions
        .handle(&ctx, &files, "extension:run-command", &[launch])
        .await
        .unwrap()
        .unwrap();
    let id = form["sessionId"].clone();
    let action = form["actions"][0]["id"].clone();
    assert_eq!(form["root"]["children"][0]["props"]["value"], "20");
    for operation in [
        "extension:refresh-session",
        "extension:search-text-changed",
        "extension:load-more",
    ] {
        let response = sessions
            .handle(&ctx, &files, operation, &[json!({"sessionId":id})])
            .await
            .unwrap()
            .unwrap();
        assert_eq!(response["mode"], "unchanged");
    }
    let copied=sessions.handle(&ctx,&files,"extension:invoke-action",&[json!({"sessionId":id,"actionId":action,"formValues":{"length":"20","useNumbers":true,"useSymbols":false}})]).await.unwrap().unwrap();
    assert_eq!(copied["mode"], "view");
    let stale=sessions.handle(&ctx,&files,"extension:invoke-action",&[json!({"sessionId":id,"actionId":action,"formValues":{"length":"20","useNumbers":true,"useSymbols":false}})]).await.unwrap().unwrap();
    assert_eq!(stale["ok"], false);
    assert_eq!(
        sessions
            .handle(
                &ctx,
                &files,
                "extension:dispose-session",
                &[json!({"sessionId":id})]
            )
            .await
            .unwrap()
            .unwrap(),
        true
    );
    assert_eq!(
        sessions
            .handle(
                &ctx,
                &files,
                "extension:refresh-session",
                &[json!({"sessionId":id})]
            )
            .await
            .unwrap()
            .unwrap()["ok"],
        false
    );
    sessions.clear().await;
}
#[tokio::test]
async fn actual_sessions_cancelled_copy_recovers_retry_without_rotation() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = context(dir.path());
    install(&ctx);
    let calls = Arc::new(Mutex::new(Vec::<Vec<Value>>::new()));
    let record = calls.clone();
    let entered = Arc::new(tokio::sync::Notify::new());
    let notify = entered.clone();
    ctx.set_router(Arc::new(move |channel, args| {
        assert_eq!(channel, "clipboard:write");
        let first = {
            let mut r = record.lock().unwrap();
            r.push(args);
            r.len() == 1
        };
        let notify = notify.clone();
        Box::pin(async move {
            if first {
                notify.notify_one();
                std::future::pending().await
            } else {
                Ok(json!({"ok":true}))
            }
        })
    }))
    .unwrap();
    let sessions = native::NativeSessions::default();
    let files = Arc::new(Mutex::new(()));
    let form=sessions.handle(&ctx,&files,"extension:run-command",&[json!({"extensionId":"com.tezbar.native.password-generator","commandName":"generate-memorable-password"})]).await.unwrap().unwrap();
    let id = form["sessionId"].clone();
    let body = [
        json!({"sessionId":id,"actionId":form["actions"][0]["id"],"formValues":{"format":"{word:capitalize}-{word}-{number:3}"}}),
    ];
    {
        let future = sessions.handle(&ctx, &files, "extension:invoke-action", &body);
        tokio::pin!(future);
        tokio::select! {_=entered.notified()=>{},_= &mut future=>panic!("copy should be pending")}
    }
    let retry = sessions
        .handle(
            &ctx,
            &files,
            "extension:refresh-session",
            &[json!({"sessionId":id})],
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retry["actions"][0]["title"], "Retry Copy");
    let again = sessions
        .handle(
            &ctx,
            &files,
            "extension:search-text-changed",
            &[json!({"sessionId":id})],
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(again["actions"], retry["actions"]);
    assert_eq!(calls.lock().unwrap().len(), 1);
    let result = sessions
        .handle(
            &ctx,
            &files,
            "extension:invoke-action",
            &[json!({"sessionId":id,"actionId":retry["actions"][0]["id"]})],
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result["ok"], true);
    {
        let record = calls.lock().unwrap();
        assert_eq!(record.len(), 2);
        assert_eq!(record[0], record[1]);
    }
    sessions.clear().await;
}
