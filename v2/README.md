# Password Generator — Rust Native V2

This package implements the three commands from **almatkai/password-generator**, own commit `ff1ceaa91365533c9ed262c4a4c0cced9aea467d`. The own source is the behavior source. The older Raycast source supplies only original icon, manifest attribution, and license evidence. V1 source, dependency declarations, and lock remain untouched.

| Command | Native behavior |
| --- | --- |
| Generate Random Password | Form defaults length `20`, numbers/symbols `true`; complete submit generates and copies, with selected group guarantees and source alphabets. |
| Copy Random Password | Automatically generates and copies **on launch**; preference defaults `length: "20"`, `useNumbers: true`, `useSymbols: true`. A failure can expose an explicit same-password Retry Copy. |
| Generate Memorable Password | Form default `{word:capitalize}-{word}-{word}-{number:3}`; JS-trimmed blank format falls back to that default; submit generates and copies. |

The command names, descriptions, preference names/defaults, and schema-2 manifest come from the own package. There are no command arguments; NativeSessions supplies `payload.argumentValues` and `payload.preferences`, and only preferences are used. `storePasswordLength`, `rememberCharacterGroups`, and `rememberCustomFormat` control **settings-only** persistence. Disabling a preference restores the source default for the next session without deleting previously saved settings. No legacy storage format is guessed or migrated.

## Behavior and limits

The exact alphabets are `abcdefghijkmnopqrstuvwxyz`, `ABCDEFGHJKLMNPQRSTUVWXYZ`, `23456789`, and `!@#$%^&*`. Random generation preserves the own order of mandatory group picks, filling picks, and Fisher–Yates decisions. The full 2,048-word English dictionary and order are extracted from locked **@scure/bip39 2.4.0**, not substituted from an upstream generator. The parser retains ASCII JS `\w`, numeric radix/exponent quirks permitted by that regex, unmatched token literals, JS trim's whitespace set, and UTF-16 length limits. Native JSON strings must be valid Unicode; malformed/unpaired surrogate JSON is rejected by Rust/host JSON parsing before the adapter.

Normal builds link **getrandom 0.2.17** for OS-backed entropy. Bounded choice accepts an even multiple of the upper bound from the 32-bit source interval, rejects the remainder, and fails closed on entropy failure. There is no predictable fallback. Rejection is capped at 128 attempts, and duplicate-word work at 32,768 draws per call; this deliberately fails closed where the JS implementation could loop indefinitely. These are implementation and finite-vector checks, not a universal parity or password-strength claim.

The own source's `hideAfterCopy` means **return to root**, with HUD options `clearRootSearch: false` and `PopToRootType.Suspended`. The pinned beta has no faithful renderer root-return contract. This package uses an honest supported adaptation: successful copies remain on the form, using a confirmation without the password when this preference is true; false uses the source-style confirmation containing the password. The no-view command returns a no-view confirmation. No `window:hide`, invented HUD/popToRoot effect, automatic retry, or extra confirmation is sent. Actual root-return/HUD parity remains an integration gap; automatic root navigation would require a separately reviewed host callback/ack change.

Only the actual SDK route `clipboard:write` with `[text]` is requested. The pinned launcher's `Worker::handle` returns `{"ok":true}` after `write_text`, and CoreContext forwards names unchanged. Success requires an object with `ok: true`, no error (absent or null), and `success` absent or true. Null, booleans, contradictory/malformed replies, and transport errors retain a same-password retry; raw service errors are never exposed. No hide/root signal occurs before or after an uncertain acknowledgment.

The secret and copy intent are checkpointed synchronously before any settings or clipboard await. A form cancellation after the OS boundary can recover the same checkpoint on refresh. Refresh/search/load-more create no entropy, IDs, settings writes, or copy effects. Checkpoints are AEAD-sealed using ChaCha20Poly1305 0.10.1 and a process-only key, with unique counter nonces and session binding; raw adapter state dumps contain ciphertext rather than a password. Process-random capability prefixes plus checked monotonic counters reject stale, forged, and cross-session actions; generated output is authoritative, ignoring supplied copy text. Successful copy clears the checkpoint, disposal clears session state, and temporary plaintext buffers use zeroize 1.9.0. This is best-effort ephemeral memory handling, not a guarantee about allocator, clipboard, renderer, crash dump, or OS memory erasure.

A **cancelled no-view launch** may already have copied across the OS boundary and lose its checkpoint when NativeSessions cleans up the unregistered entry. Reopening is a new generation; the package does not add a normal confirmation step or global secret map to conceal that limitation. The extension creates no password history/cache, logs, or telemetry. Tezbar's separate clipboard watcher/OS clipboard managers can independently capture clipboard contents; this port does not change or claim to disable them. Only settings use the SDK store; its process-atomic transaction preserves unknown keys and revalidates malformed JSON and typed corruption inside the write transaction. It provides no cross-process transaction guarantee.

## Integration, without editing the host

`integration/vendor-map.json` fixes the stable layout:

- Vendor the package under `src-tauri/native-extensions/password-generator/`, excluding `.work/`, `target/`, and `artifacts/`.
- The sibling module is `#[path = "../../../native-extensions/password-generator/src/adapter.rs"] mod v2_password_generator;`, so `super::v2_sdk` resolves to the actual host SDK.
- Merge the dependency entries in `integration/Cargo.dependencies.toml.fixture` into the existing host manifest and lock.
- Copy `icons/password-generator.png` to `src/renderer/public/extensions/icons/password-generator.png` **byte for byte**.
- Review `integration/registration.patch`: additive slug/command/handle dispatch, manifest append, compiled catalog count 11 → 12, and module registration against beta `fd300e276356587726566026f072563eb9c34cb1`.
- On the committed host `fe71ca87f21ac4ff31505010c228f8abda0f2a89`, review the separate `icon-prerequisite-fe71ca8.patch`. It permits only this canonical ID's exact bundled icon URL, preserving ordinary file checks and all prior ports. It is tested against the actual current registry source.

These are review artifacts, not an instruction to replace a newer host catalog or drop intervening ports. Reconcile the additive changes into the parent's current host, retain all previous entries/counts, rebuild, install `com.tezbar.native.password-generator`, and verify actual compiled CLI coverage. JSON cannot load this implementation: it must be compiled into the host. The canonical manifest replaces `raycast.password-generator`; actual registry alias handling prefers installed compiled native commands and leaves the V1 package available for fallback. Merely copying the manifest or a native marker cannot supply execution.

## Reproducible verification

Rust tests are offline with cached crates:

```sh
cargo test --manifest-path v2/Cargo.toml --locked --offline
cargo clippy --manifest-path v2/Cargo.toml --locked --offline --all-targets -- -D warnings
cargo +1.85.0 test --manifest-path v2/Cargo.toml --locked --offline
cargo +1.85.0 clippy --manifest-path v2/Cargo.toml --locked --offline --all-targets -- -D warnings
```

Run `v2/scripts/reproduce.sh` for isolated own V1 tests/typecheck, oracle regeneration, real React Form/Surface tests, both Rust toolchains, Windows GNU check/Clippy, and full Draft 2020-12 schema validation with negative cases. Dependency setup needs Node/pnpm/npm/Python and network or their caches; it writes only disposable `v2/.work/`. `scripts/package-check.py` audits and extracts the actual Cargo archive, then reruns locked offline tests and strict Clippy without host/source absolute paths. Windows commands compile-check only; no Windows executable runs.

Evidence includes **72 actual TS engine vectors** (outputs/errors and every bounded random choice/range), **18 executable TS/TSX command scenarios**, **3 engine tests + 11 integration tests**, and actual pinned FormRuntime/ExtensionRuntimeSurface tests for defaults, Checkbox/Dropdown collection, complete submission maps, callbacks as `formValues.value`, and queued draft preservation. Submit-only fields retain their initial server values; no controlled replacement protocol or guessed hotkey/tooltip support is introduced. Deterministic bounded-choice RNGs and a public-byte `getrandom` dependency seam exist only in test targets. Production constructors always select the real locked entropy crate. The compile-only `examples/beta-adapter-check.rs` also compiles the actual adapter with `cfg(test)` disabled and the real entropy dependency; it is never run.

The beta harness compiles the real SDK, CoreContext, NativeSessions, registry, catalog and additive dispatch. Its only source transform disables unrelated upstream `cfg(test)` blocks; runtime function bodies are retained. Tauri event emission, entropy bytes, and execution of unrelated adapters are explicit test seams. Existing catalog commands of other adapters are inventory stubs, **not evidence of their compiled execution**. React tests run actual components with chrome/OS hooks and unrelated surfaces stubbed, not a browser. No real clipboard, focus, paste, selection, user profile, or host was exercised.

Own V1 tests (3) and TypeScript typecheck pass in a disposable copy with its unchanged pnpm lock. Initial pnpm 11 installation reported ignored esbuild build scripts (exit 1); installed dependencies worked for direct tests/typecheck. Reproduction explicitly disables install scripts in the disposable copy and does not repair V1 or its lock. No legacy-peer-deps workaround was needed.

Host registration/rebuild/install, actual macOS/Windows forms, Windows entropy API execution, and clipboard/root/focus/keyboard QA are **NOT-RUN**. Both manifest verification records remain `not-run`. Packaging/integration tests do not establish host installation or OS QA.

## Evidence and credits

Original Raycast pin: `6734d85992290066fd31a7f5bee3cca3d3545a1a`, `extensions/password-generator`. Author **joshuaiz**; contributors **teziovsky, pernielsentikaer, hnsn, yusifaliyevpro, xmok, peduarte**. The immutable original manifest and repository-root MIT license are in `fixtures/original/` (there is no extension-local LICENSE at that pin). The own root `assets/icon.svg` stays unchanged.

Exact original icon: **17,803 bytes**, SHA-256 **`7d9f69d316d24698d8dd643a8f2b4a0532b2a71065eb4096a7d39864e1529518`**. It is not recolored, optimized, or redrawn.

The actual locked @scure/bip39 MIT license credits **Patricio Palladino and Paul Miller (paulmillr.com)** and is retained with the complete source English list and npm tarball. Its SRI is verified against the own pnpm lock. Cargo checksums, selected entropy/checkpoint library source archives, and each locked crate's available license notices are retained under `fixtures/library/`; no separate grant is fabricated. Node oracle source/license evidence comes from the installed Node v26.7.0 runtime. `PROVENANCE.json` records source pins and `INDEX.json` records deliverable byte counts/SHA-256. Rehash the index after any reviewed edits and again after staging.

XERJ used canonical root prefix `ax-13fb795fab97`, generation 1: 12 files, 6/6 source files indexed, 76 records. Its initial source coverage was complete; later generated/vendored artifacts are verified exhaustively by current files, byte indexes and archive checks rather than relying on search coverage. The final reindex aborted after scanning 279 files: the frozen dataset schema rejected `v2/Cargo.lock` and required unsupported schema evolution. No new prefix, schema bypass, or server restart was attempted. New V2 search coverage is incomplete; generation 1 still covers the original six own source files.
